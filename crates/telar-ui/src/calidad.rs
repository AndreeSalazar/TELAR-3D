//! Controles discretos del muestreo gráfico, sin acceder a Vulkan.
use crate::{Distribucion, Estudio, Objetivo, Rectangulo, lienzo::Lienzo};
use alloc::format;

impl Estudio {
    pub(crate) fn dibujar_calidad(&self, lienzo: &mut Lienzo, diseno: Distribucion) {
        let primera = diseno.calidad(0);
        let estilo = self.estilo("etiqueta", false, false);
        let texto = format!("MSAA {}x", self.salida.muestras_reales);
        lienzo.etiqueta(Rectangulo { x: diseno.pausa().x, y: primera.y, ancho: 82.0, alto: 24.0 }, &texto, &estilo);
        for (indice, (muestras, texto)) in [(1, "1x"), (2, "2x"), (4, "4x")].into_iter().enumerate() {
            let estilo = self.estilo(
                "calidad",
                self.encima == Some(Objetivo::Calidad(indice)),
                self.salida.muestras == muestras,
            );
            let caja = diseno.calidad(indice);
            lienzo.caja(caja, &estilo);
            lienzo.etiqueta(caja.contenido(&estilo), texto, &estilo);
        }
    }
}
