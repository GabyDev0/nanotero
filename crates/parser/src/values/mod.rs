use std::collections::BTreeMap;

/// Representa cualquier tipo de dato válido en el formato de configuración Tero v0.1.
///
/// Este enum es la estructura final de datos en memoria que devuelve el parser.
/// Contiene directamente los valores primitivos o las colecciones anidadas
/// extraídas del archivo `.tero`.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Ausencia intencional de valor (corresponde a `nil` en Tero).
    Nil,

    /// Un valor lógico (`true` o `false`).
    Boolean(bool),

    /// Un número de punto flotante de doble precisión (`f64`), incluyendo `NaN` e `Infinity`.
    Float(f64),

    /// Un número entero con signo de 64 bits (`i64`).
    Integer(i64),

    /// Una cadena de texto codificada en UTF-8.
    String(String),

    /// Una lista indexada y ordenada de elementos de tipo [`Value`].
    Array(Vec<Value>),

    /// Una colección ordenada de pares clave-valor, mapeada mediante un [`BTreeMap`].
    Object(BTreeMap<String, Value>),
}
