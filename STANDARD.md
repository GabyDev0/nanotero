# Tero v0.1

Este documento define la sintaxis oficial, las reglas de parseo y el comportamiento esperado para los archivos de configuración **.tero**.

## 1. Sistema de Tipos y Valores

**Tero** clasifica sus componentes en dos categorías: **Valores Primitivos** (datos inmutables básicos) y **Estructuras de Datos** (contenedores complejos).

### 1.1 Valores Primitivos

| Tipo | Sintaxis / Representación | Descripción Técnica |
| --- | --- | --- |
| **Entero** | `42`, `-1024` | Almacenado nativamente como entero con signo de 64 bits (`i64`). |
| **Flotante** | `3.1416`, `-0.005` | Almacenado como número de punto flotante de doble precisión (`f64`). |
| **Booleano** | `true` o `false` | Valores lógicos tradicionales para control de flujo o banderas. |
| **Cadena (String)** | `"texto"` | Secuencia de caracteres UTF-8 envuelta obligatoriamente en comillas dobles. |
| **Nil** | `nil` | Representa la ausencia intencional de cualquier valor o un estado no inicializado. |

#### Constantes Numéricas Especiales

Tero reconoce dos palabras clave heredadas del estándar IEEE 754 para operaciones matemáticas excepcionales:

* `NaN`: (*Not a Number*) Resultado de una operación matemática inválida o indeterminada.
* `Infinity`: Representa un valor numérico que desborda el límite del tipo `f64`.

---

### 1.2 Estructuras de Datos (Objetos y Colecciones)

Las estructuras de datos permiten agrupar valores primitivos u otras sub-estructuras, soportando anidación profunda.

#### Objetos `{}`

Colecciones ordenadas de pares clave-valor. Las claves dentro de un objeto deben ser únicas en su nivel de jerarquía.

```tero
configuracion: {
    activo: true,
    intentos: 3
}

```

#### Arreglos (Arrays) `[]`

Listas indexadas y ordenadas de valores. Pueden contener tipos de datos mixtos de forma nativa.

```tero
puertos_permitidos: [80, 443, 8080],
estado_servidores: [true, "en_espera", nil]
```

## 2. Declaración de Versión

**Tero** permite especificar bajo qué versión del estándar ha sido diseñado el archivo de configuración.

### 2.1 Estructura y Reglas

* **Ubicación:** Si se incluye, debe declararse **estrictamente en la primera línea** del archivo.
* **Sintaxis:** Se compone de la palabra clave `Tero` (respetando la mayúscula inicial), seguida de exactamente un espacio en blanco y una constante numérica (`Flotante`). Debe finalizar inmediatamente con un salto de línea (`\n`).
* **Opcionalidad:** La declaración de versión es **opcional**. Si se omite, el *parser* asumirá por defecto la versión más reciente que soporte la librería cliente.

### Ejemplo de Uso

```text
Tero 0.1
Usuarios [{
    Nombre: "GabyDev0"
}]
```

## 3. Estructuras de Datos: Objetos `{}`

Un objeto es una colección indexada y ordenada de pares clave-valor. Las claves dentro de un mismo objeto deben ser únicas en su nivel de jerarquía.

### 3.1 Identificadores de Clave

Las claves se declaran sin comillas (estilo texto plano). Deben comenzar obligatoriamente con una letra o un guion bajo (`_`) y solo pueden contener caracteres alfanuméricos (`[a-zA-Z0-9_]`).

### 3.2 Separador de Asignación

* **Regla General:** Se utiliza estrictamente el carácter de dos puntos (`:`) para separar la clave de su valor.
* **Excepción de Estructura:** Si el valor asignado es una estructura de datos compleja (**Objeto** o **Arreglo**), el uso de los dos puntos es **completamente opcional**.

### 3.3 Separador de Elementos y Comas

* **Delimitador:** Los pares clave-valor dentro de un objeto se separan mediante una coma (`,`) **o** mediante un salto de línea (`\n`).
* **Coma Terminal (Trailing Comma):** La coma después del último elemento de un objeto es opcional y completamente válida.

### Ejemplos de Sintaxis

```tero
# Variante 1: En una sola línea, usando comas y asignación opcional en objeto anidado
usuario { nombre: "GabyDev0", rol: "admin" }

# Variante 2: Estilo bloque, usando saltos de línea (sin comas ni dos puntos)
servidor {
    ip: "127.0.0.1"
    puerto: 8080
    
    # El formato permite mezclar la omisión de dos puntos en estructuras
    modulos [ "auth", "api", "db" ]
}

```

## 4. Estructuras de Datos: Arreglos (Arrays) `[]`

Un arreglo es una lista ordenada e indexada de valores.

### 4.1 Reglas de Sintaxis

* **Multitipo:** Un arreglo puede contener cualquier combinación de tipos de datos de forma nativa (enteros, flotantes, cadenas, booleanos, objetos, `nil` u otros arreglos).
* **Separador de Elementos:** Al igual que los objetos, los elementos dentro de un arreglo se pueden separar mediante una coma (`,`) **o** mediante un salto de línea (`\n`).
* **Coma Terminal (Trailing Comma):** Permitida y opcional después del último elemento.

### Ejemplos de Sintaxis

```tero
# Variante 1: En una sola línea separados por comas
puertos: [80, 443, 8080]

# Variante 2: Multilinea usando saltos de línea (sin comas)
servidores [
    "api.gabydev0.com"
    "auth.gabydev0.com"
]

# Variante 3: Arreglo mixto con comas terminales
datos: [
    "produccion",
    true,
    100,
]
```

## 5. Comentarios

**Tero** permite incluir anotaciones y texto explicativo dentro del archivo utilizando el carácter almohadilla o *hashtag* (`#`).

### 5.1 Reglas de Parseo

* **Ignorados por el Parser:** Todo el texto que se encuentre a la derecha de un carácter `#` es completamente ignorado por la librería lectora y no tiene ningún impacto en el objeto final en memoria.
* **Ubicación:** Los comentarios pueden ocupar una línea completa o colocarse al final de una línea de código (comentarios *inline*).

### Ejemplos de Sintaxis

```tero
# Este es un comentario de línea completa para documentar el bloque
servidor {
    puerto: 8080 # Comentario inline: Puerto por defecto para la API
    seguro: true
}

```
