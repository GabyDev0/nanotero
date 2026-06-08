use std::fmt::Display;

/// Representa los tipos de datos primitivos que nanoloom puede inferir.
#[derive(Debug, PartialEq, Clone)]
pub enum Literal<'a> {
    Integer(i64),
    Float(f64),
    Text(&'a str), // contiene \.
    RawText(&'a str), // texto directo 
    Boolean(bool),
    Nil,
    NaN,
    Infinity
}


/// El núcleo dinámico de datos de Tero.
///
/// **Nota de diseño:** En la versión v0.1.0, este enum actúa únicamente como un contenedor 
/// directo para [`Literal`], por lo que su estructura actual parece redundante. Sin embargo, 
/// está diseñado como un punto de extensión crítico para futuras versiones, donde pasará de 
/// almacenar solo datos planos a soportar la evaluación de expresiones matemáticas complejas 
/// (ej. `SPEED * ACCELERATION`) directamente en tiempo de análisis.
#[derive(Debug, PartialEq, Clone)]
pub enum LoomValue<'a> {
    /// Un valor literal plano y directo (número, cadena, booleano, etc.).
    Raw(Literal<'a>),
    
    // Guarda: (Operador, Izquierda, Derecha) -> ej: SPEED * ACCELERATION
    // Expression(Operator, Box<LoomValue<'a>>, Box<LoomValue<'a>>),
}


/// Representa las unidades léxicas individuales (tokens) emitidas por el Lexer.
///
/// Este enum vincula los caracteres de control de la sintaxis con referencias a porciones
/// del texto original (`&'a str`) para evitar asignaciones de memoria innecesarias.
#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    /// Un valor primitivo ya procesado por el lexer (números, booleanos, cadenas, etc.).
    Value(LoomValue<'a>),

    /// Un nombre de clave o identificador que apunta a una sección del código fuente.
    Identifier(&'a str),

    /* Caracteres de Control */

    /// Dos puntos (`:`), utilizados para separar claves de sus valores.
    Colon,

    /// Coma (`,`), utilizada como delimitador opcional o separador de elementos.
    Comma,

    /// Llave de apertura (`{`), indica el inicio de un bloque de objeto.
    LeftBrace,

    /// Llave de cierre (`}`), indica el fin de un bloque de objeto.
    RightBrace,

    /// Corchete de apertura (`[`), indica el inicio de un arreglo o lista.
    LeftBracket,

    /// Corchete de cierre (`]`), indica el fin de un arreglo o lista.
    RightBracket,

    /// Salto de línea (`\n`), utilizado para la separación implícita de declaraciones.
    Newline,

    /// Operador de sustracción o signo menos (`-`), utilizado para denotar números negativos.
    Sub,
}
impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Colon => write!(f,":"),
            Self::Comma => write!(f,","),
            Self::Sub => write!(f,"-"),
            Self::LeftBrace => write!(f,"{{"),
            Self::LeftBracket => write!(f,"["),
            Self::RightBrace => write!(f,"}}"),
            Self::RightBracket => write!(f,"]"),
            Self::Newline => write!(f,"\\n"),
            Self::Identifier(v) => write!(f,"{}",*v),
            Self::Value(v) => write!(f, "{}", v)
        }
    }
}
impl Display for LoomValue<'_>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Raw(Literal::Boolean(b)) => write!(f, "{}", b),
            Self::Raw(Literal::Float(fl)) => write!(f, "{}", fl),
            Self::Raw(Literal::Integer(i)) => write!(f, "{}", i),
            Self::Raw(Literal::Nil) => write!(f, "nil"),
            Self::Raw(Literal::NaN) => write!(f, "{}", f64::NAN),
            Self::Raw(Literal::Infinity) => write!(f, "{}", f64::INFINITY),
            Self::Raw(Literal::RawText(text)) => write!(f, "{}", *text),
            Self::Raw(Literal::Text(text)) => write!(f, "{}", *text)
        }
    }
}