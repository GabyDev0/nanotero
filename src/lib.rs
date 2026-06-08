use nanotero_parser::{parser_string};
pub use nanotero_parser::{Value, LoomError};
pub use nanotero_lex::LexError;

/// Interpreta una cadena de texto en formato Tero y la convierte en un árbol de datos [`Value`].
///
/// # Errores
///
/// Devuelve un [`LoomError`] si la cadena de texto contiene errores léxicos o sintácticos
/// que no cumplan con el estándar de Tero v0.1.
///
/// # Ejemplo
///
/// ```rust
///
/// let codigo = "config { activo: true }";
/// let resultado = nanotero::from_str(codigo);
/// assert!(resultado.is_ok());
/// ```
#[inline(always)]
pub fn from_str(code: &str) -> Result<Value, LoomError> {
    parser_string(code)
}
    
