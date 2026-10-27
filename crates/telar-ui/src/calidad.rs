//! Controles discretos del muestreo gráfico, sin acceder a Vulkan.
use crate::{Distribucion, Estudio, Objetivo, lienzo::Lienzo, numero::Campo};
use alloc::format;

impl Estudio {
    pub(crate) fn dibujar_calidad(&self, lienzo: &mut Lienzo, diseno: Distribucion) {
        let estilo = self.estilo_numero(Campo::Muestras);
        let texto = format!("MSAA {}x", self.salida.muestras);
        let texto = self.texto_numero(Campo::Muestras, &texto);
        let rect = diseno.numero_muestras();
        lienzo.caja(rect, &estilo);
        lienzo.etiqueta(rect.interior(4.0), &texto, &estilo);
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
