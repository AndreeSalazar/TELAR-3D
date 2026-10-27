//! Cobertura de bordes: el MSAA genera niveles intermedios sin desenfocar interiores.
#![allow(clippy::unwrap_used, clippy::expect_used, reason = "fallos explícitos de verificación")]
use telar_gpu::{
    Calculo, CapaInterfaz, Constantes, KernelsProcedurales, Opciones, ParametrosProcedurales, Sombreadores,
};

fn capturar(antialias: bool) -> telar_gpu::CapturaEstudio {
    let gpu = Calculo::nuevo(&Opciones { antialias, ..Opciones::default() }).expect("Vulkan real");
    let codigo = [telar_kernel::compilar(), hilo_spirv::rejilla::generar(), hilo_spirv::rejilla::deformar()];
    let mut residente = gpu
        .procedural(2, KernelsProcedurales { preparar: &codigo[0], generar: &codigo[1], deformar: &codigo[2] })
        .unwrap();
    residente.paso(ParametrosProcedurales { lado: 2, fase: 0.0, amplitud: 0.0 }).unwrap();
    let escena = [hilo_spirv::visor::vertices(), hilo_spirv::visor::fragmentos()];
    let ui = [hilo_spirv::interfaz::vertices(), hilo_spirv::interfaz::fragmentos()];
    let constantes = Constantes {
        matriz: [[0.2, 0.1, 0.0, 0.0], [0.0; 4], [0.1, 0.3, 0.0, 0.0], [0.0, 0.0, 0.5, 1.0]],
        luz: [0.0, 1.0, 0.0],
        color: [1.0; 4],
        cielo: [0.0; 4],
        suelo: [0.0; 4],
    };
    let resultado = residente
        .medir_estudio(
            Sombreadores { vertices: &escena[0], fragmentos: &escena[1] },
            (64, 64),
            [0.0, 0.0, 0.0, 1.0],
            &constantes.plana(telar_gpu::FORMATO_CAPTURA),
            CapaInterfaz {
                shaders: Sombreadores { vertices: &ui[0], fragmentos: &ui[1] },
                vertices: &[],
                area: [0, 0, 64, 64],
                atlas: None,
            },
        )
        .unwrap();
    assert_eq!(residente.actividad().geometria_subida, 0);
    resultado
}

#[test]
fn bordes_con_cobertura_parcial_y_centro_nitido_en_msaa_real() {
    let sin = capturar(false);
    let con = capturar(true);
    assert_eq!(sin.muestras, 1);
    assert!(matches!(con.muestras, 1 | 2 | 4));
    let maximo = sin.pixeles.as_chunks::<4>().0.iter().map(|p| p[0]).max().unwrap();
    assert!(maximo > 200, "la superficie debe estar visible y uniformemente iluminada");
    let intermedios = |pixeles: &[u8]| pixeles.as_chunks::<4>().0.iter().filter(|p| p[0] > 0 && p[0] < maximo).count();
    assert_eq!(intermedios(&sin.pixeles), 0);
    if con.muestras > 1 {
        assert!(intermedios(&con.pixeles) > 30, "el borde necesita cobertura fraccionaria");
        assert_ne!(sin.pixeles, con.pixeles);
    } else {
        assert_eq!(sin.pixeles, con.pixeles, "fallback 1x explícito");
    }
    let centro = (32 * 64 + 32) * 4;
    assert_eq!(&sin.pixeles[centro..centro + 4], &con.pixeles[centro..centro + 4]);
    assert_eq!(con.pixeles[0], 0, "el suavizado no debe alcanzar el fondo distante");
    assert_eq!(telar_gpu::avisos_de_validacion(), 0);
}
