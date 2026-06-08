# Contribuir a NanoTero 🐦

¡Gracias por tu interés en mejorar NanoTero! Toda ayuda es bienvenida, ya sea reportando un bug, sugiriendo una característica o mejorando la documentación.

---

## 🐛 Cómo reportar un problema

Si encuentras un error o el parser falla con una sintaxis válida:

1. Revisa la pestaña de **Issues** para ver si alguien ya lo ha reportado.
2. Si no es así, abre un nuevo Issue describiendo:
   * El código `.tero` que causó el fallo.
   * El error exacto que devolvió la librería (léxico o sintáctico).
   * El comportamiento que esperabas.

---

## 🛠️ Cómo proponer cambios (Pull Requests)

Si quieres meterle mano al código y programar una mejora, sigue estos pasos:

1. Haz un **Fork** del repositorio a tu propia cuenta.
2. Crea una rama para tu característica:

```bash
   git checkout -b feat/nueva-caracteristica
```

*(O `fix/nombre-del-bug` si estás corrigiendo un error).*
3. Escribe tu código respetando las reglas del proyecto (ver abajo).
4. Asegúrate de añadir **tests** que demuestren que tu cambio funciona.
5. Abre un **Pull Request** hacia la rama `main` del repositorio original.

---

## 📜 Reglas del Código (Estilo Rust)

Para mantener la base de código limpia y eficiente, tu código debe cumplir lo siguiente antes de enviar el PR:

* **Formato:** Pasa siempre el formateador oficial para que el estilo sea idéntico al resto del proyecto:

```bash
cargo fmt
```

* **Calidad:** Ejecuta el linter de Rust para asegurarte de que no hay malas prácticas o advertencias:

```bash
cargo clippy
```

* **Estabilidad:** Todos los tests existentes (y los nuevos que añadas) deben pasar sin errores:

```bash
cargo test
```

¡Gracias por hacer de NanoTero un parser más rápido y estable para todos! 🚀
