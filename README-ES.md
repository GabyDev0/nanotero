# NanoTero 🐦

**NanoTero** es una librería ultra-minimalista, ligera y de alto rendimiento diseñada para deserializar archivos de configuración `.tero` directamente en estructuras seguras de Rust.

Diseñada bajo una arquitectura *zero-copy*, un enfoque descendente recursivo y operaciones SIMD en su núcleo léxico, ofrece un análisis sintáctico predecible y eficiente sin saturar la memoria.

---

## 🚀 Características Clave

* **API Minimalista:** Un único punto de entrada para deserializar tus datos.
* **Type-Safe Deserialization:** Mapea el texto directamente a structs nativos de Rust usando macros de derivación.
* **Manejo de Errores Tipado:** Control absoluto sobre fallos léxicos, sintácticos y de evaluación.
* **Alto Rendimiento:** Procesamiento de strings optimizado mediante operaciones vectoriales (*branchless* SIMD) y validación exacta de UTF-8.

---

## 🛠️ API Pública (Todo lo que necesitas)

NanoTero mantiene su superficie de API al mínimo para garantizar la máxima velocidad de compilación y facilidad de uso. La librería exporta únicamente los componentes esenciales:

### 1. `nanotero::from_str`

Es la función principal y el motor de la librería. Recibe un string con el código fuente de Tero y reconstruye cualquier tipo que implemente el trait `Deserialize`.

```rust
#[inline(always)]
pub fn from_str<T: Deserialize>(code: &str) -> Result<T, TeroError>;

```

### 2. `#[derive(TeroDeserialize)]`

El macro de derivación por excelencia. Te permite marcar tus structs para que puedan ser autogenerados por el parser. Soporta el atributo auxiliar `#[tero(name = "...")]` para renombrar propiedades cuando el archivo de configuración no coincide exactamente con el identificador de Rust.

```rust
#[derive(TeroDeserialize)]
struct Config {
    #[tero(name = "server_ip")]
    ip: String,
    port: u32,
}

```

### 3. `nanotero::TeroError`

El enum unificado que gestiona todo el flujo de fallos del sistema. Te dice exactamente si el proceso falló en la fase léxica, sintáctica o durante el mapeo de tipos en la evaluación.

### 4. `nanotero::Deserialize`

El trait que gobierna la conversión de tipos. Implementado de forma nativa para primitivos (`i64`, `f64`, `bool`, `String`), envoltorios (`Box`, `Cow`, `Option`), y colecciones estándar (`BTreeMap`).

---

## 🎨 Ejemplo de Uso

Aquí tienes un ejemplo completo de cómo integrar **NanoTero** en tu proyecto de Rust usando la nueva API fuertemente tipada:

```rust
use nanotero::{from_str, TeroDeserialize, TeroError};

// Definimos la estructura esperada de la configuración
#[derive(TeroDeserialize, Debug)]
struct ServidorConfig {
    // Mapeamos "server_name" en el archivo .tero al campo "nombre"
    #[tero(name = "server_name")]
    nombre: String,
    puerto: u32,
    activo: bool,
    modos: Vec<String>,
}

fn main() {
    // Tu cadena de texto en formato Tero
    let codigo_config = r#"
        server_name: "KiwiServer"
        puerto: 8080
        activo: true
        modos [ "produccion", "debug" ] # Comentario integrado
    "#;

    // Deserialización directa al struct indicado mediante inferencia de tipos
    let result = from_str::<ServidorConfig>(codigo_config).unwrap();
}
```

---

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Para más detalles, consulta el archivo `LICENSE`.