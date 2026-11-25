# EcoPuntos CR

EcoPuntos CR es un contrato inteligente de Stellar Soroban para una campaña de reciclaje comunitaria en Costa Rica. Cuando un centro de acopio lleva los registros en papel o en varias hojas de cálculo, puede costar saber cuántos puntos le quedan a cada persona y cuántos kilos reunió la campaña.

Elegí este problema porque una campaña puede empezar con un solo centro de acopio y tres materiales. El centro pesa el material y confirma la entrega. El contrato aplica la misma tarifa a cada participante, actualiza los saldos y permite consultar cuánto se ha reciclado.

## Cómo funciona

El administrador se fija al desplegar el contrato y debe autorizar el constructor. Solo esa dirección puede registrar entregas. Las cantidades se expresan en kilogramos enteros y los saldos en puntos enteros.

| Material en Rust | Puntos por kg |
| --- | ---: |
| `Material::Plastico` | 10 |
| `Material::Papel` | 5 |
| `Material::Aluminio` | 20 |

| Función | Qué hace | Autorización |
| --- | --- | --- |
| `registrar_reciclaje(participante, material, kg)` | Acredita puntos, suma los kilos a ambos totales y devuelve los puntos otorgados. | Administrador |
| `saldo(participante)` | Devuelve el saldo de una dirección. | Consulta pública |
| `canjear(participante, puntos)` | Descuenta puntos y devuelve el saldo restante. | Titular del saldo |
| `total_por_material(material)` | Devuelve los kilos acumulados para ese material. | Consulta pública |
| `total_reciclado()` | Devuelve los kilos acumulados de todos los materiales. | Consulta pública |

Por ejemplo, si Ana entrega 2 kg de plástico y 3 kg de papel, recibe 35 puntos y el total de la campaña aumenta en 5 kg. Si canjea 7 puntos, le quedan 28 y los totales de reciclaje se conservan.

Los saldos y los totales empiezan en cero. El contrato rechaza cantidades de cero, canjes mayores al saldo y operaciones que excedan el rango de `u64`. El centro de acopio verifica el pesaje, evita registrar una entrega dos veces y gestiona la entrega del premio fuera del contrato. Cada llamada de registro autorizada representa una nueva entrega. Los puntos son internos de la campaña.

## Compilar y ejecutar las pruebas

Instala [Rust con rustup](https://rustup.rs/). El archivo `rust-toolchain.toml` fija Rust 1.92.0 e incluye el destino `wasm32v1-none`. La primera ejecución descarga la herramienta y las dependencias. En Linux también se necesita un enlazador para las pruebas nativas, como el de `build-essential` en Debian o Ubuntu.

Desde la raíz del proyecto:

```sh
cargo build --locked --target wasm32v1-none --release --lib
cargo test --locked
```

El primer comando genera `target/wasm32v1-none/release/ecopuntos_cr.wasm`. El segundo ejecuta `src/test.rs` en un entorno local de Soroban. Para compilar y probar basta con Cargo, sin cuenta de Stellar ni conexión a una red Stellar.

## Qué prueban los tests

- Las consultas empiezan en cero.
- Las entregas acumulan kilos por material y mantienen un saldo por persona.
- La autorización del participante por sí sola no permite registrar reciclaje.
- El canje requiere la autorización del titular, permite gastar todo el saldo y conserva los totales.
- Los registros y canjes de cero y los canjes sin saldo suficiente se rechazan.
- Un desbordamiento deja los saldos y los totales intactos.
