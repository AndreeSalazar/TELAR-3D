//! Los números de SPIR-V que emite TELAR: códigos de operación, capacidades,
//! clases de almacenamiento y decoraciones. Copiados de `spirv.h`
//! (`spirv/unified1`, SDK de Vulkan 1.4.350) y comprobados uno a uno contra él.

/// La palabra mágica con la que empieza todo módulo SPIR-V.
pub const MAGIA: u32 = 0x0723_0203;
/// SPIR-V 1.3: la clase `StorageBuffer` ya es del núcleo, y lo acepta Vulkan 1.1+.
pub const VERSION_1_3: u32 = 0x0001_0300;

// Códigos de operación (SpvOp*).
/// `OpName`.
pub const NAME: u16 = 5;
/// `OpMemoryModel`.
pub const MEMORY_MODEL: u16 = 14;
/// `OpEntryPoint`.
pub const ENTRY_POINT: u16 = 15;
/// `OpExecutionMode`.
pub const EXECUTION_MODE: u16 = 16;
/// `OpCapability`.
pub const CAPABILITY: u16 = 17;
/// `OpTypeVoid`.
pub const TYPE_VOID: u16 = 19;
/// `OpTypeBool`.
pub const TYPE_BOOL: u16 = 20;
/// `OpTypeInt`.
pub const TYPE_INT: u16 = 21;
/// `OpTypeVector`.
pub const TYPE_VECTOR: u16 = 23;
/// `OpTypeRuntimeArray`.
pub const TYPE_RUNTIME_ARRAY: u16 = 29;
/// `OpTypeStruct`.
pub const TYPE_STRUCT: u16 = 30;
/// `OpTypePointer`.
pub const TYPE_POINTER: u16 = 32;
/// `OpTypeFunction`.
pub const TYPE_FUNCTION: u16 = 33;
/// `OpConstant`.
pub const CONSTANT: u16 = 43;
/// `OpFunction`.
pub const FUNCTION: u16 = 54;
/// `OpFunctionEnd`.
pub const FUNCTION_END: u16 = 56;
/// `OpVariable`.
pub const VARIABLE: u16 = 59;
/// `OpLoad`.
pub const LOAD: u16 = 61;
/// `OpStore`.
pub const STORE: u16 = 62;
/// `OpAccessChain`.
pub const ACCESS_CHAIN: u16 = 65;
/// `OpArrayLength`.
pub const ARRAY_LENGTH: u16 = 68;
/// `OpDecorate`.
pub const DECORATE: u16 = 71;
/// `OpMemberDecorate`.
pub const MEMBER_DECORATE: u16 = 72;
/// `OpIAdd`.
pub const I_ADD: u16 = 128;
/// `OpIMul`.
pub const I_MUL: u16 = 132;
/// `OpISub`.
pub const I_SUB: u16 = 130;
/// `OpUDiv`.
pub const U_DIV: u16 = 134;
/// `OpUMod`.
pub const U_MOD: u16 = 137;
/// `OpConvertUToF`.
pub const U_A_F: u16 = 112;
/// `OpConvertFToU`.
pub const F_A_U: u16 = 109;
/// `OpBitcast`.
pub const BITCAST: u16 = 124;
/// `OpSelect`.
pub const SELECT: u16 = 169;
/// `OpINotEqual`.
pub const I_NOT_EQUAL: u16 = 171;
/// `OpIEqual`.
pub const I_EQUAL: u16 = 170;
/// `OpULessThanEqual`.
pub const U_LESS_THAN_EQUAL: u16 = 178;
/// `OpLogicalAnd`.
pub const LOGICAL_AND: u16 = 167;
/// `OpFDiv`.
pub const F_DIV: u16 = 136;
/// `OpFOrdGreaterThan`.
pub const F_ORD_GREATER_THAN: u16 = 186;
/// `OpULessThan`.
pub const U_LESS_THAN: u16 = 176;
/// `OpShiftRightLogical`.
pub const SHIFT_RIGHT_LOGICAL: u16 = 194;
/// `OpShiftLeftLogical`.
pub const SHIFT_LEFT_LOGICAL: u16 = 196;
/// `OpBitwiseXor`.
pub const BITWISE_XOR: u16 = 198;
/// `OpBitwiseAnd`.
pub const BITWISE_AND: u16 = 199;
/// `OpSelectionMerge`.
pub const SELECTION_MERGE: u16 = 247;
/// `OpLabel`.
pub const LABEL: u16 = 248;
/// `OpBranch`.
pub const BRANCH: u16 = 249;
/// `OpBranchConditional`.
pub const BRANCH_CONDITIONAL: u16 = 250;
/// `OpReturn`.
pub const RETURN: u16 = 253;

// Enumeraciones.
/// `Capability Shader`.
pub const CAPACIDAD_SHADER: u32 = 1;
/// `AddressingModel Logical`.
pub const DIRECCIONES_LOGICAS: u32 = 0;
/// `MemoryModel GLSL450`.
pub const MEMORIA_GLSL450: u32 = 1;
/// `ExecutionModel GLCompute`.
pub const MODELO_CALCULO: u32 = 5;
/// `ExecutionMode LocalSize`.
pub const MODO_TAMANO_GRUPO: u32 = 17;
/// `Decoration Block`.
pub const DECORACION_BLOQUE: u32 = 2;
/// `Decoration ArrayStride`.
pub const DECORACION_PASO: u32 = 6;
/// `Decoration BuiltIn`.
pub const DECORACION_INTEGRADA: u32 = 11;
/// `Decoration Binding`.
pub const DECORACION_ENLACE: u32 = 33;
/// `Decoration DescriptorSet`.
pub const DECORACION_CONJUNTO: u32 = 34;
/// `Decoration Offset`.
pub const DECORACION_DESPLAZAMIENTO: u32 = 35;
/// `BuiltIn GlobalInvocationId`.
pub const INTEGRADA_ID_GLOBAL: u32 = 28;
/// `FunctionControl None` y `SelectionControl None`.
pub const CONTROL_NINGUNO: u32 = 0;

// Para los shaders de gráficos (vértices y fragmentos).
/// `OpExtInstImport`.
pub const EXT_INST_IMPORT: u16 = 11;
/// `OpExtInst`.
pub const EXT_INST: u16 = 12;
/// `OpTypeFloat`.
pub const TYPE_FLOAT: u16 = 22;
/// `OpVectorShuffle`.
pub const VECTOR_SHUFFLE: u16 = 79;
/// `OpCompositeConstruct`.
pub const COMPOSITE_CONSTRUCT: u16 = 80;
/// `OpCompositeExtract`.
pub const COMPOSITE_EXTRACT: u16 = 81;
/// `OpFAdd`.
pub const F_ADD: u16 = 129;
/// `OpFSub`.
pub const F_SUB: u16 = 131;
/// `OpFMul`.
pub const F_MUL: u16 = 133;
/// `OpVectorTimesScalar`.
pub const VECTOR_TIMES_SCALAR: u16 = 142;
/// `OpDot`.
pub const DOT: u16 = 148;
/// `ExecutionModel Vertex`.
pub const MODELO_VERTICES: u32 = 0;
/// `ExecutionModel Fragment`.
pub const MODELO_FRAGMENTOS: u32 = 4;
/// `ExecutionMode OriginUpperLeft`: el origen de los fragmentos arriba a la izquierda (obligatorio en Vulkan).
pub const MODO_ORIGEN_ARRIBA: u32 = 7;
/// `Decoration Location`.
pub const DECORACION_UBICACION: u32 = 30;
/// `BuiltIn Position`.
pub const INTEGRADA_POSICION: u32 = 0;
/// `GLSLstd450Normalize` (de `GLSL.std.450.h`).
pub const GLSL_NORMALIZAR: u32 = 69;
/// `GLSLstd450FAbs`.
pub const GLSL_ABSOLUTO: u32 = 4;
/// `GLSLstd450FMax`.
pub const GLSL_MAXIMO: u32 = 40;
/// `GLSLstd450FMix`.
pub const GLSL_MEZCLA: u32 = 46;

/// Dónde vive una variable (`StorageClass`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Clase {
    /// `Input`: lo que llega del sistema (`gl_GlobalInvocationID`).
    Entrada = 1,
    /// `Output`: lo que el shader entrega a la etapa siguiente (`gl_Position`, el color).
    Salida = 3,
    /// `Function`: variables locales.
    Funcion = 7,
    /// `PushConstant`: unos pocos bytes que la CPU manda con cada dibujo (la cámara).
    Constantes = 9,
    /// `StorageBuffer`: un buffer de la GPU que el kernel lee y escribe.
    Almacen = 12,
}

impl Clase {
    /// Su número en SPIR-V.
    #[must_use]
    pub const fn numero(self) -> u32 {
        self as u32
    }
}
