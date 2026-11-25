use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    IntoVal,
};

fn iniciar() -> (Env, Address, Address) {
    let env = Env::default();
    let administrador = Address::generate(&env);
    env.mock_all_auths();
    let contrato = env.register(EcoPuntosCr, (&administrador,));
    env.set_auths(&[]);
    (env, contrato, administrador)
}

#[test]
fn consultas_iniciales_en_cero() {
    let (env, contrato, _) = iniciar();
    let cliente = EcoPuntosCrClient::new(&env, &contrato);
    assert_eq!(cliente.saldo(&Address::generate(&env)), 0);
    assert_eq!(cliente.total_reciclado(), 0);
    for material in [Material::Plastico, Material::Papel, Material::Aluminio] {
        assert_eq!(cliente.total_por_material(&material), 0);
    }
}

#[test]
fn acumula_por_material_y_mantiene_saldos_separados() {
    let (env, contrato, administrador) = iniciar();
    let cliente = EcoPuntosCrClient::new(&env, &contrato);
    let ana = Address::generate(&env);
    let luis = Address::generate(&env);

    let autorizaciones = [MockAuth {
        address: &administrador,
        invoke: &MockAuthInvoke {
            contract: &contrato,
            fn_name: "registrar_reciclaje",
            args: (&ana, Material::Plastico, 2_u64).into_val(&env),
            sub_invokes: &[],
        },
    }];
    let autorizado = cliente.mock_auths(&autorizaciones);
    assert_eq!(
        autorizado.registrar_reciclaje(&ana, &Material::Plastico, &2),
        20
    );

    let autorizado = cliente.mock_all_auths();
    assert_eq!(
        autorizado.registrar_reciclaje(&ana, &Material::Papel, &3),
        15
    );
    assert_eq!(
        autorizado.registrar_reciclaje(&luis, &Material::Aluminio, &1),
        20
    );
    assert_eq!(
        autorizado.registrar_reciclaje(&luis, &Material::Plastico, &4),
        40
    );
    assert_eq!(cliente.saldo(&ana), 35);
    assert_eq!(cliente.saldo(&luis), 60);
    assert_eq!(cliente.total_por_material(&Material::Plastico), 6);
    assert_eq!(cliente.total_por_material(&Material::Papel), 3);
    assert_eq!(cliente.total_por_material(&Material::Aluminio), 1);
    assert_eq!(cliente.total_reciclado(), 10);
}

#[test]
fn rechaza_registro_firmado_solo_por_el_participante() {
    let (env, contrato, _) = iniciar();
    let cliente = EcoPuntosCrClient::new(&env, &contrato);
    let participante = Address::generate(&env);
    let autorizaciones = [MockAuth {
        address: &participante,
        invoke: &MockAuthInvoke {
            contract: &contrato,
            fn_name: "registrar_reciclaje",
            args: (&participante, Material::Papel, 1_u64).into_val(&env),
            sub_invokes: &[],
        },
    }];
    let sin_permiso = cliente.mock_auths(&autorizaciones);
    assert!(sin_permiso
        .try_registrar_reciclaje(&participante, &Material::Papel, &1)
        .is_err());
    assert_eq!(cliente.saldo(&participante), 0);
    assert_eq!(cliente.total_reciclado(), 0);
}

#[test]
fn canje_requiere_al_titular_y_conserva_los_totales() {
    let (env, contrato, administrador) = iniciar();
    let cliente = EcoPuntosCrClient::new(&env, &contrato);
    let participante = Address::generate(&env);
    cliente
        .mock_all_auths()
        .registrar_reciclaje(&participante, &Material::Aluminio, &1);

    let autorizaciones = [MockAuth {
        address: &administrador,
        invoke: &MockAuthInvoke {
            contract: &contrato,
            fn_name: "canjear",
            args: (&participante, 7_u64).into_val(&env),
            sub_invokes: &[],
        },
    }];
    let ajeno = cliente.mock_auths(&autorizaciones);
    assert!(ajeno.try_canjear(&participante, &7).is_err());
    assert_eq!(cliente.saldo(&participante), 20);

    let autorizaciones = [MockAuth {
        address: &participante,
        invoke: &MockAuthInvoke {
            contract: &contrato,
            fn_name: "canjear",
            args: (&participante, 7_u64).into_val(&env),
            sub_invokes: &[],
        },
    }];
    let titular = cliente.mock_auths(&autorizaciones);
    assert_eq!(titular.canjear(&participante, &7), 13);
    assert_eq!(cliente.mock_all_auths().canjear(&participante, &13), 0);
    assert_eq!(cliente.saldo(&participante), 0);
    assert_eq!(cliente.total_por_material(&Material::Aluminio), 1);
    assert_eq!(cliente.total_reciclado(), 1);
}

#[test]
fn rechaza_cantidades_en_cero_y_saldo_insuficiente() {
    let (env, contrato, _) = iniciar();
    let cliente = EcoPuntosCrClient::new(&env, &contrato);
    let autorizado = cliente.mock_all_auths();
    let participante = Address::generate(&env);
    assert_eq!(
        autorizado.try_registrar_reciclaje(&participante, &Material::Papel, &0),
        Err(Ok(Error::CantidadCero))
    );
    autorizado.registrar_reciclaje(&participante, &Material::Papel, &1);
    assert_eq!(
        autorizado.try_canjear(&participante, &0),
        Err(Ok(Error::CantidadCero))
    );
    assert_eq!(
        autorizado.try_canjear(&participante, &6),
        Err(Ok(Error::SaldoInsuficiente))
    );
    assert_eq!(cliente.saldo(&participante), 5);
    assert_eq!(cliente.total_reciclado(), 1);
}

#[test]
fn rechaza_desbordamientos_sin_modificar_saldos_ni_totales() {
    let (env, contrato, _) = iniciar();
    let cliente = EcoPuntosCrClient::new(&env, &contrato);
    let autorizado = cliente.mock_all_auths();
    let participante = Address::generate(&env);
    assert_eq!(
        autorizado.try_registrar_reciclaje(&participante, &Material::Papel, &u64::MAX),
        Err(Ok(Error::Desbordamiento))
    );
    let kg = u64::MAX / 5;
    let puntos = autorizado.registrar_reciclaje(&participante, &Material::Papel, &kg);
    assert_eq!(
        autorizado.try_registrar_reciclaje(&participante, &Material::Papel, &1),
        Err(Ok(Error::Desbordamiento))
    );
    assert_eq!(cliente.saldo(&participante), puntos);
    assert_eq!(cliente.total_por_material(&Material::Papel), kg);
    assert_eq!(cliente.total_reciclado(), kg);
}
