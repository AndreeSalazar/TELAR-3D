# TELAR estilo

Hoja de estilos para la interfaz nativa. `no_std + alloc`, sin dependencias,
Windows, Vulkan ni estado de escena. Se inspira en el [modelo de cajas CSS del
W3C](https://www.w3.org/TR/css-box-3/); implementa un subconjunto explícito.

El tema de toda la vista vive en [temas/telar.css](temas/telar.css). Se analiza
una vez al crear el estudio. La resolución produce valores tipados que usa
`telar-ui` para distribuir cajas y componer triángulos; `telar-gpu` conserva su
aislamiento y solo recibe geometría, colores y atlas. No hay navegador ni HTML.

## Reglas disponibles

```css
:root { --acento: #80cfbe; }
.nodo {
    background-color: #272c37;
    border-color: #495163;
    border-width: 1px;
    border-radius: 6px;
    padding: 12px;
    font-size: 14px;
    line-height: 20px;
}
.nodo:hover { border-color: var(--acento); }
.nodo:checked { background-color: #263c3c; }
```

- Selectores de una clase ASCII: `.nodo`, con `:hover`, `:checked`, `:focus` y
  `:active` combinables. El compositor actual conecta hover y selección;
  `:checked` representa nodo/material seleccionado o pausa activada.
- Cascada por número de estados y orden de origen; a igual especificidad gana
  la última declaración. Herencia explícita de color, tamaño y altura de línea.
- Variables en `:root`, `var(--nombre)` y comentarios `/* ... */`.
- Colores opacos `#rrggbb`; longitudes no negativas en `px`, o `0` sin unidad.
- `background-color`, `color`, `border-color`, `border-width`, `border-radius`,
  `padding`, `margin`, `gap`, `width`, `height`, `min-width`, `max-width`,
  `font-size`, `line-height`, `box-shadow`.
- `width` y `height` admiten `auto`. Sombra única sólida: `x y 0 color`, sin
  desenfoque, o `none`; solo los desplazamientos de sombra admiten negativos.
- `Caja::contenido` descuenta borde y padding. Las medidas son `border-box`.

## Personalizar sin recompilar

```powershell
cargo run -p telar-app --release -- --estilo mi-tema.css
```

El archivo contiene personalizaciones que se añaden al tema incorporado. Se lee
al arrancar; no hay recarga automática todavía. Un error se informa y no aplica
una hoja parcial. La API `Estudio::cargar_estilos` conserva el tema anterior si falla.

## Alcance del compositor

Las propiedades se consumen por componente: `.estudio` define fondo, relleno y
separación exterior; `.inspector`, ancho y alto lateral; `.panel`, cajas interiores;
`.cabecera`, `.barra` y `.pie`, alturas; `.nodo`, tarjetas y tipografía;
`.control`, pista y separación entre deslizadores; `.boton` y `.muestra`, acciones.
`.visor` controla limpieza y borde rectangular del área 3D. `.valor`, `.etiqueta`,
`.titulo`, `.puerto`, `.hilo` y `.puntos` completan el lenguaje visual.

El compositor aplica límites de tamaño para mantener utilizable una ventana de
720×600. No todas las propiedades cambian el layout de cualquier componente:
la distribución de paneles sigue siendo una composición nativa especializada.
No implementa DOM, selectores descendientes, media queries, flex/grid, porcentajes,
animaciones, fuentes CSS, shorthands múltiples, `!important` ni CSS web completo.
La sintaxis no implementada produce error en lugar de ignorarse silenciosamente.

Las pruebas verifican cascada, variables y ciclos, herencia, cajas, errores y el
tema incluido; las pruebas UI/GPU comprueban que la hoja altera dibujo e interacción.
