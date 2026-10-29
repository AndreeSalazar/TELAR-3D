//! Reloj host opcional: medir llamadas y esperas sin cambiar su política.
use std::time::Instant;
pub(super) struct Cronometro {
    inicio: Option<Instant>,
    fases: [f64; 6],
}
impl Cronometro {
    pub(super) fn nuevo(activo: bool) -> Self {
        Self { inicio: activo.then(Instant::now), fases: [0.0; 6] }
    }
    pub(super) fn marcar(&mut self, fase: usize) {
        if let Some(anterior) = self.inicio {
            let ahora = Instant::now();
            self.fases[fase] = ahora.duration_since(anterior).as_secs_f64() * 1000.0;
            self.inicio = Some(ahora);
        }
    }
    pub(super) fn resultado(&self, cuadros: u64) -> Option<crate::TelemetriaHost> {
        self.inicio.map(|_| crate::TelemetriaHost {
            cuadros,
            preparacion_ms: None,
            valla_ms: self.fases[0],
            observacion_ms: self.fases[1],
            adquirir_ms: self.fases[2],
            grabar_ms: self.fases[3],
            enviar_ms: self.fases[4],
            presentar_ms: self.fases[5],
        })
    }
}
