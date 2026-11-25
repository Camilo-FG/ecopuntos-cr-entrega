#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Material {
    Plastico,
    Papel,
    Aluminio,
}

impl Material {
    fn puntos_por_kg(self) -> u64 {
        match self {
            Self::Plastico => 10,
            Self::Papel => 5,
            Self::Aluminio => 20,
        }
    }
}

#[contracttype]
#[derive(Clone)]
enum Clave {
    Administrador,
    Saldo(Address),
    TotalMaterial(Material),
    TotalGeneral,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    CantidadCero = 1,
    SaldoInsuficiente = 2,
    Desbordamiento = 3,
}

#[contract]
pub struct EcoPuntosCr;

#[contractimpl]
impl EcoPuntosCr {
    pub fn __constructor(env: Env, administrador: Address) {
        administrador.require_auth();
        env.storage()
            .instance()
            .set(&Clave::Administrador, &administrador);
    }

    /// Acredita puntos por kilogramos enteros y devuelve los puntos otorgados.
    pub fn registrar_reciclaje(
        env: Env,
        participante: Address,
        material: Material,
        kg: u64,
    ) -> Result<u64, Error> {
        let administrador: Address = env.storage().instance().get(&Clave::Administrador).unwrap();
        administrador.require_auth();

        if kg == 0 {
            return Err(Error::CantidadCero);
        }

        let puntos = kg
            .checked_mul(material.puntos_por_kg())
            .ok_or(Error::Desbordamiento)?;

        let datos = env.storage().persistent();
        let clave_saldo = Clave::Saldo(participante);
        let clave_material = Clave::TotalMaterial(material);
        let saldo_actual: u64 = datos.get(&clave_saldo).unwrap_or(0);
        let material_actual: u64 = datos.get(&clave_material).unwrap_or(0);
        let total_actual: u64 = datos.get(&Clave::TotalGeneral).unwrap_or(0);

        let nuevo_saldo = saldo_actual
            .checked_add(puntos)
            .ok_or(Error::Desbordamiento)?;
        let nuevo_material = material_actual
            .checked_add(kg)
            .ok_or(Error::Desbordamiento)?;
        let nuevo_total = total_actual.checked_add(kg).ok_or(Error::Desbordamiento)?;

        datos.set(&clave_saldo, &nuevo_saldo);
        datos.set(&clave_material, &nuevo_material);
        datos.set(&Clave::TotalGeneral, &nuevo_total);
        Ok(puntos)
    }

    pub fn saldo(env: Env, participante: Address) -> u64 {
        env.storage()
            .persistent()
            .get(&Clave::Saldo(participante))
            .unwrap_or(0)
    }

    /// Descuenta los puntos autorizados por el participante y devuelve el saldo restante.
    pub fn canjear(env: Env, participante: Address, puntos: u64) -> Result<u64, Error> {
        participante.require_auth();
        if puntos == 0 {
            return Err(Error::CantidadCero);
        }

        let datos = env.storage().persistent();
        let clave = Clave::Saldo(participante);
        let saldo_actual: u64 = datos.get(&clave).unwrap_or(0);
        let restante = saldo_actual
            .checked_sub(puntos)
            .ok_or(Error::SaldoInsuficiente)?;
        datos.set(&clave, &restante);
        Ok(restante)
    }

    pub fn total_por_material(env: Env, material: Material) -> u64 {
        env.storage()
            .persistent()
            .get(&Clave::TotalMaterial(material))
            .unwrap_or(0)
    }

    pub fn total_reciclado(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&Clave::TotalGeneral)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test;
