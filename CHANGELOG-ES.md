# Registro de Cambios

Todos los cambios notables en este proyecto serán documentados en este archivo.

El formato está basado en [Keep a Changelog](https://keepsachangelog.com/en/1.0.0/),
y este proyecto se adhiere a [Versionado Semántico](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-06-27

### Añadido
- **Macro Derive `TeroDeserialize`:** Se introdujo una macro procedimental central (`#[derive(TeroDeserialize)]`) para mapear automáticamente los campos de configuración `.tero` directamente a estructuras Rust con seguridad de tipos.
- **Atributo `#[tero(name = "...")]`:** Soporte para renombrar campos personalizados durante la derivación de la macro.
- **Enum `TeroError`:** Una arquitectura de manejo de errores unificada completamente nueva que categoriza los fallos en variantes precisas: `Lexical`, `Syntactic` y `Eval`.

### Cambiado
- **Motor de API con Seguridad de Tipos:** Se refactorizó `from_str` para que retorne cualquier tipo que implemente `Deserialize` mediante inferencia de tipos (`from_str::<T>`) en lugar de forzar un árbol de datos `Value` genérico.
- **Unificación del Crate:** Se fusionaron los antiguos sub-crates internos `lex` y `parser` en una única unidad de compilación de alto rendimiento (`compiler`) para optimizar los tiempos de compilación y agilizar la gestión de estados internos.

### Corregido
- **Reporte de Diagnóstico de Errores:** Se rehizo la lógica de formateo de errores. Los errores sintácticos y lógicos ahora son completamente legibles, descriptivos y explícitos, en lugar de tragarse el contexto.
- **Lógica de Casos Límite del Parser:** Se corrigió un error crítico en la estructura de descenso recursivo que previamente causaba evaluaciones incorrectas de tokens bajo ciertas disposiciones finales específicas.

### Eliminado
- **`LoomError`:** Se eliminó por completo el antiguo y ambiguo tipo de error en favor del nuevo `TeroError` estructurado.