# NanoTero 🐦

**NanoTero** es una librería ultra-minimalista, ligera y de alto rendimiento diseñada para parsear e interactuar con archivos de configuración `.tero`.

Diseñada bajo una arquitectura *zero-copy* y un enfoque descendente recursivo, ofrece un análisis sintáctico predecible y eficiente sin saturar la memoria.

---

## 🚀 Características Clave

* **API Minimalista:** Un único punto de entrada para parsear tus datos.
* **Manejo de Errores Tipado:** Control absoluto sobre fallos léxicos y sintácticos.
* **Estructura Ligera:** Representación limpia del árbol de datos mediante un único Enum.

---

## 🛠️ API Pública (Todo lo que necesitas)

NanoTero mantiene su superficie de API al mínimo para garantizar la máxima velocidad de compilación y facilidad de uso. La librería exporta únicamente cuatro componentes esenciales:

### 1. `nanotero::from_str`

Es la función principal y el motor de la librería. Recibe un string con el código fuente de Tero y te devuelve el árbol de datos parseado.

```rust
pub fn from_str(code: &str) -> Result<Value, LoomError>;
```

### 2. `nanotero::Value`

El enum central que representa cualquier tipo de dato válido dentro del ecosistema de Tero. Almacena de forma segura estructuras anidadas y primitivos:

* `Value::Object(BTreeMap<String, Value>)`
* `Value::Array(Vec<Value>)`
* `Value::String(String)`
* `Value::Integer(i64)`
* `Value::Float(f64)`
* `Value::Boolean(bool)`
* `Value::Nil`

### 3. `nanotero::LoomError`

El enum que unifica los errores del sistema. Te dice exactamente si el parser falló por un token inesperado, un fin de archivo prematuro (`UnexpectedEOF`) o si el problema venía desde las entrañas del Lexer.

### 4. `nanotero::LexError`

El enum dedicado exclusivamente a los errores del analizador léxico (Lexer). Se activa si el usuario escribe caracteres no reconocidos, números mal formateados o cadenas sin cerrar. Viaja dentro de la variante `LoomError::Lexical(LexError)`.

---

## 🎨 Ejemplo de Uso

Aquí tienes un ejemplo completo de cómo integrar **NanoTero** en tu proyecto de Rust para leer una configuración:

```rust
use nanotero::{from_str, Value, LoomError};

fn main() {
    // Tu cadena de texto en formato Tero
    let codigo_config = r#"
        servidor: "KiwiServer",
        puerto: 8080,
        activo: true,
        modos [ "produccion", "debug" ] # Comentario integrado
    "#;

    // Parseo directo en un solo paso
    match from_str(codigo_config) {
        Ok(Value::Object(mapa)) => {
            println!("¡Configuración parseada con éxito!");
            if let Some(Value::String(nombre)) = mapa.get("servidor") {
                println!("Nombre del servidor: {}", nombre);
            }
        }
        Ok(_) => println!("Se esperaba un objeto raíz."),
        Err(LoomError::UnexpectedToken(t)) => eprintln!("Error sintáctico: Token inesperado '{}'", t),
        Err(LoomError::Lexical(lex_err)) => eprintln!("Error léxico en el archivo: {:?}", lex_err),
        Err(err) => eprintln!("Error al parsear: {:?}", err),
    }
}

```

---

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Para más detalles, consulta el archivo `LICENSE`.
