use std::collections::BTreeMap;

use acir::{
    brillig::{BinaryFieldOp, Opcode as BrilligOpcode, RegisterIndex, RegisterOrMemory, Value},
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        opcodes::{BlockId, MemOp},
        Circuit, Opcode, OpcodeLocation,
    },
    native_types::{Expression, Witness, WitnessMap},
    FieldElement,
};

use acvm::{
    pwg::{ACVMStatus, ErrorLocation, ForeignCallWaitInfo, OpcodeResolutionError, ACVM},
    BlackBoxFunctionSolver,
};
use blackbox_solver::BlackBoxResolutionError;

pub(crate) struct StubbedBackend;

impl BlackBoxFunctionSolver for StubbedBackend {
    fn schnorr_verify(
        &self,
        _public_key_x: &FieldElement,
        _public_key_y: &FieldElement,
        _signature: &[u8],
        _message: &[u8],
    ) -> Result<bool, BlackBoxResolutionError> {
        panic!("Path not trodden by this test")
    }
    fn pedersen(
        &self,
        _inputs: &[FieldElement],
        _domain_separator: u32,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        panic!("Path not trodden by this test")
    }
    fn fixed_base_scalar_mul(
        &self,
        _low: &FieldElement,
        _high: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
        panic!("Path not trodden by this test")
    }
}

// Reenable these test cases once we move the brillig implementation of inversion down into the acvm stdlib.

#[test]
fn inner_recursion() {
    let bytecode = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 237, 214, 83, 172, 29, 91, 0, 128, 225, 125, 123, 207,
        169, 113, 106, 219, 182, 109, 219, 58, 181, 109, 219, 182, 109, 219, 182, 121, 109, 245,
        90, 229, 123, 211, 127, 210, 63, 205, 121, 233, 83, 155, 52, 105, 186, 146, 47, 127, 102,
        246, 206, 204, 36, 43, 43, 107, 61, 11, 133, 66, 209, 67, 47, 71, 184, 29, 104, 63, 66, 52,
        124, 140, 48, 127, 15, 254, 27, 3, 49, 17, 11, 177, 17, 7, 113, 17, 15, 241, 145, 0, 17,
        72, 136, 68, 72, 140, 36, 72, 138, 100, 72, 142, 20, 72, 137, 84, 72, 141, 52, 72, 139,
        116, 72, 143, 12, 200, 136, 76, 200, 140, 44, 200, 138, 108, 200, 142, 28, 200, 137, 92,
        200, 141, 60, 200, 139, 124, 200, 143, 2, 40, 136, 66, 40, 140, 34, 40, 138, 98, 40, 142,
        18, 40, 137, 82, 40, 141, 50, 40, 139, 114, 40, 143, 10, 168, 136, 74, 168, 140, 42, 168,
        138, 106, 168, 142, 26, 168, 137, 90, 168, 141, 58, 168, 139, 122, 168, 143, 6, 104, 136,
        70, 104, 140, 38, 104, 138, 102, 104, 142, 22, 104, 137, 86, 104, 141, 54, 104, 139, 72,
        180, 67, 123, 116, 64, 71, 116, 66, 103, 116, 65, 87, 116, 67, 119, 244, 64, 79, 244, 66,
        111, 244, 65, 95, 244, 67, 127, 12, 112, 46, 195, 157, 203, 72, 231, 117, 16, 6, 99, 8,
        134, 98, 24, 134, 99, 4, 70, 98, 20, 70, 99, 12, 198, 98, 28, 198, 99, 2, 38, 98, 18, 38,
        99, 10, 166, 98, 26, 166, 99, 6, 102, 98, 22, 102, 99, 14, 230, 98, 30, 230, 99, 1, 22, 98,
        17, 22, 99, 9, 150, 98, 25, 150, 99, 5, 86, 98, 21, 86, 99, 13, 214, 98, 29, 214, 99, 3,
        54, 98, 19, 54, 99, 11, 182, 98, 27, 182, 99, 7, 118, 98, 23, 118, 99, 15, 246, 98, 31,
        246, 227, 0, 14, 226, 16, 14, 227, 8, 142, 226, 24, 142, 227, 4, 78, 226, 20, 78, 227, 12,
        206, 226, 28, 206, 227, 2, 46, 226, 18, 46, 227, 10, 174, 226, 26, 174, 227, 6, 110, 226,
        22, 110, 227, 14, 238, 226, 30, 238, 59, 15, 209, 156, 139, 96, 124, 226, 189, 96, 125, 69,
        120, 239, 51, 124, 142, 47, 240, 37, 190, 194, 215, 248, 6, 223, 226, 59, 124, 143, 31,
        240, 35, 126, 194, 207, 248, 5, 15, 124, 118, 176, 14, 163, 174, 225, 231, 190, 247, 185,
        239, 10, 26, 102, 195, 109, 116, 27, 195, 198, 180, 177, 108, 108, 27, 199, 198, 181, 241,
        108, 124, 155, 192, 70, 216, 132, 54, 145, 77, 108, 147, 216, 164, 54, 153, 77, 110, 83,
        216, 148, 54, 149, 77, 109, 211, 216, 180, 54, 157, 77, 111, 51, 216, 140, 54, 147, 205,
        108, 179, 216, 172, 54, 155, 205, 110, 115, 216, 156, 54, 151, 205, 109, 243, 216, 188, 54,
        159, 205, 111, 11, 216, 130, 182, 144, 45, 108, 139, 216, 162, 182, 152, 45, 110, 75, 216,
        146, 182, 148, 45, 109, 203, 216, 178, 182, 156, 45, 111, 43, 216, 138, 182, 146, 173, 108,
        171, 216, 170, 182, 154, 173, 110, 107, 216, 154, 182, 150, 173, 109, 235, 216, 186, 182,
        158, 173, 111, 27, 216, 134, 182, 145, 109, 108, 155, 216, 166, 182, 153, 109, 110, 91,
        216, 150, 182, 149, 109, 109, 219, 216, 182, 54, 210, 182, 179, 237, 109, 7, 219, 209, 118,
        178, 157, 109, 23, 219, 213, 118, 179, 221, 109, 15, 219, 211, 246, 178, 189, 109, 31, 219,
        215, 246, 179, 253, 237, 0, 59, 48, 202, 119, 6, 99, 144, 215, 131, 237, 16, 59, 212, 14,
        179, 195, 237, 8, 59, 210, 142, 178, 163, 237, 24, 59, 214, 142, 179, 227, 237, 4, 59, 209,
        78, 178, 147, 237, 20, 59, 213, 78, 179, 211, 237, 12, 59, 211, 206, 178, 179, 237, 28, 59,
        215, 206, 179, 243, 237, 2, 187, 208, 46, 178, 139, 237, 18, 187, 212, 46, 179, 203, 237,
        10, 187, 210, 174, 178, 171, 237, 26, 187, 214, 174, 179, 235, 237, 6, 187, 209, 110, 178,
        155, 237, 22, 187, 213, 110, 179, 219, 237, 14, 187, 211, 238, 178, 187, 237, 30, 187, 215,
        238, 179, 251, 237, 1, 123, 208, 30, 178, 135, 237, 17, 123, 212, 30, 179, 199, 237, 9,
        123, 210, 158, 178, 167, 237, 25, 123, 214, 158, 179, 231, 237, 5, 123, 209, 94, 178, 151,
        237, 21, 123, 213, 94, 179, 215, 237, 13, 123, 211, 222, 178, 183, 237, 29, 123, 215, 222,
        179, 247, 109, 212, 61, 47, 184, 254, 212, 190, 218, 248, 126, 197, 111, 248, 29, 127, 224,
        79, 252, 133, 191, 241, 15, 254, 197, 127, 248, 31, 15, 241, 8, 143, 241, 4, 79, 67, 47,
        55, 178, 176, 183, 248, 188, 7, 62, 231, 195, 33, 248, 253, 56, 4, 127, 56, 244, 190, 155,
        67, 111, 176, 224, 131, 197, 254, 166, 7, 219, 215, 141, 23, 26, 153, 198, 164, 193, 14, 0,
        0,
    ];
    let circuit = Circuit::read(&*bytecode).unwrap();

    fn hex_to_field(hex_str: &str) -> FieldElement {
        FieldElement::from_hex(hex_str).unwrap()
    }

    let witness_assignments: WitnessMap = BTreeMap::from([
        (
            Witness(1),
            hex_to_field("0x21082ca216cbbf4e1c6e4f4594dd508c996dfbe1174efb98b11509c6e306460b"),
        ),
        (
            Witness(2),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000010"),
        ),
        (
            Witness(3),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000005"),
        ),
        (
            Witness(4),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000010"),
        ),
        (
            Witness(5),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000001"),
        ),
        (
            Witness(6),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(7),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(8),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(9),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(10),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(11),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(12),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(13),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(14),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(15),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(16),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(17),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(18),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(19),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(20),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(21),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(22),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(23),
            hex_to_field("0x0000000000000000000000000000004cf4015c3a5297f556c3b72581f2dca64d"),
        ),
        (
            Witness(24),
            hex_to_field("0x00000000000000000000000000000000000a67b44462aa65528a3e3b181e0bcd"),
        ),
        (
            Witness(25),
            hex_to_field("0x00000000000000000000000000000091507f347e13f13eec9d9f327ac25ada11"),
        ),
        (
            Witness(26),
            hex_to_field("0x00000000000000000000000000000000001993946f40247aa93aedba75857f3e"),
        ),
        (
            Witness(27),
            hex_to_field("0x0000000000000000000000000000005d340a5ecb1a33c0b7055734ef91200c97"),
        ),
        (
            Witness(28),
            hex_to_field("0x00000000000000000000000000000000001eebbe1207643a8bd1669b999e8226"),
        ),
        (
            Witness(29),
            hex_to_field("0x0000000000000000000000000000006b27d5d1ffba12754d0718481e1a9a419a"),
        ),
        (
            Witness(30),
            hex_to_field("0x00000000000000000000000000000000002f08a6a07ed616c588bcf4e3555c00"),
        ),
        (
            Witness(31),
            hex_to_field("0x0000000000000000000000000000003cbc8e573c1299e8ba491bd2218a413bd7"),
        ),
        (
            Witness(32),
            hex_to_field("0x0000000000000000000000000000000000192b586ec6fb3b1b6d063a00a86c65"),
        ),
        (
            Witness(33),
            hex_to_field("0x000000000000000000000000000000c4516b3cffabe3dcdd074d74f595c81c04"),
        ),
        (
            Witness(34),
            hex_to_field("0x000000000000000000000000000000000021142418da600cf97a5248cbd45524"),
        ),
        (
            Witness(35),
            hex_to_field("0x000000000000000000000000000000c292117b1a17fefe9de0bfd9edf1a84bf9"),
        ),
        (
            Witness(36),
            hex_to_field("0x000000000000000000000000000000000002d6fd9e84dbe74b7531e1801405a1"),
        ),
        (
            Witness(37),
            hex_to_field("0x000000000000000000000000000000459a3b2a0b768da45ac7af7f2aec40fc42"),
        ),
        (
            Witness(38),
            hex_to_field("0x0000000000000000000000000000000000293c6ab3c06a0669af13393a82c60a"),
        ),
        (
            Witness(39),
            hex_to_field("0x0000000000000000000000000000006c845044cca9a2d9dbf94039a11d999aaa"),
        ),
        (
            Witness(40),
            hex_to_field("0x00000000000000000000000000000000000efe5ad29f99fce939416b6638dff2"),
        ),
        (
            Witness(41),
            hex_to_field("0x000000000000000000000000000000587f768022c11ac8e37cd9dce243d01ef2"),
        ),
        (
            Witness(42),
            hex_to_field("0x00000000000000000000000000000000000a44bf49517a4b66ae6b51eee6ac68"),
        ),
        (
            Witness(43),
            hex_to_field("0x00000000000000000000000000000059d49ef10107e88711fc0919e244e17a08"),
        ),
        (
            Witness(44),
            hex_to_field("0x0000000000000000000000000000000000132d159fdf1907b0619b9809904594"),
        ),
        (
            Witness(45),
            hex_to_field("0x00000000000000000000000000000016d9bd1186bcef7a31846ce703eb4cb5b2"),
        ),
        (
            Witness(46),
            hex_to_field("0x0000000000000000000000000000000000291c00ed4a7689fec327330104b85c"),
        ),
        (
            Witness(47),
            hex_to_field("0x0000000000000000000000000000004b6c55389300451eb2a2deddf244129e7a"),
        ),
        (
            Witness(48),
            hex_to_field("0x000000000000000000000000000000000018c3e78f81e83b52719158e4ac4c2f"),
        ),
        (
            Witness(49),
            hex_to_field("0x0000000000000000000000000000008d7beb75f905a5894e18d27c42c62fd797"),
        ),
        (
            Witness(50),
            hex_to_field("0x00000000000000000000000000000000000002e9c902fe5cd49b64563cadf3bb"),
        ),
        (
            Witness(51),
            hex_to_field("0x0000000000000000000000000000000d9e28aa6d00e046852781a5f20816645c"),
        ),
        (
            Witness(52),
            hex_to_field("0x00000000000000000000000000000000002cbce7beee3076b78dace04943d69d"),
        ),
        (
            Witness(53),
            hex_to_field("0x000000000000000000000000000000fd915d11bfedbdc0e59de09e5b28952080"),
        ),
        (
            Witness(54),
            hex_to_field("0x00000000000000000000000000000000002bc27ec2e1612ea284b08bcc55b6f2"),
        ),
        (
            Witness(55),
            hex_to_field("0x000000000000000000000000000000be6ed4f4d252a79059e505f9abc1bdf3ed"),
        ),
        (
            Witness(56),
            hex_to_field("0x00000000000000000000000000000000000ad34b5e8db72a5acf4427546c7294"),
        ),
        (
            Witness(57),
            hex_to_field("0x00000000000000000000000000000090a049f42a3852acd45e6f521f24b4900e"),
        ),
        (
            Witness(58),
            hex_to_field("0x00000000000000000000000000000000001e5b26790a26eb340217dd9ad28dbf"),
        ),
        (
            Witness(59),
            hex_to_field("0x000000000000000000000000000000ac27e570ae50bc180509764eb3fef94815"),
        ),
        (
            Witness(60),
            hex_to_field("0x0000000000000000000000000000000000155a0f51fec78c33ffceb7364d69d7"),
        ),
        (
            Witness(61),
            hex_to_field("0x000000000000000000000000000000b644999713a8d3c66e9054aa5726324c76"),
        ),
        (
            Witness(62),
            hex_to_field("0x00000000000000000000000000000000001c1c4720bed44a591d97cbc72b6e44"),
        ),
        (
            Witness(63),
            hex_to_field("0x000000000000000000000000000000058cc5ad51753faec2a5908155d472e429"),
        ),
        (
            Witness(64),
            hex_to_field("0x00000000000000000000000000000000000f7261cf55a71f4d0d7b961dda9ddb"),
        ),
        (
            Witness(65),
            hex_to_field("0x0000000000000000000000000000004a36df78f0d50144437ef26f8bbfe69ac1"),
        ),
        (
            Witness(66),
            hex_to_field("0x00000000000000000000000000000000001b7b1a10c1e638ce11d8c84b831aca"),
        ),
        (
            Witness(67),
            hex_to_field("0x000000000000000000000000000000826ba5b1d1ddd8d6bb960f01cd1321a169"),
        ),
        (
            Witness(68),
            hex_to_field("0x0000000000000000000000000000000000163a9c8b67447afccc64e9ccba9d9e"),
        ),
        (
            Witness(69),
            hex_to_field("0x0000000000000000000000000000007653a773088aba5c6b1337f435188d72c4"),
        ),
        (
            Witness(70),
            hex_to_field("0x000000000000000000000000000000000019256311d43dbc795f746c63b20966"),
        ),
        (
            Witness(71),
            hex_to_field("0x000000000000000000000000000000df58a7bad9afe3651be67bc6c298092e11"),
        ),
        (
            Witness(72),
            hex_to_field("0x00000000000000000000000000000000001fa51a0d75363b3af4e259e0dbb2c5"),
        ),
        (
            Witness(73),
            hex_to_field("0x000000000000000000000000000000c8b5836b29551d41dbc04bdb1fcf1a1868"),
        ),
        (
            Witness(74),
            hex_to_field("0x000000000000000000000000000000000021915198840ad9c3666122b2837aea"),
        ),
        (
            Witness(75),
            hex_to_field("0x0000000000000000000000000000005df0e69d7efdbc7898b3762f0a0ed976ad"),
        ),
        (
            Witness(76),
            hex_to_field("0x00000000000000000000000000000000000cee6b75dcf02a07c50939e8ca3cf3"),
        ),
        (
            Witness(77),
            hex_to_field("0x00000000000000000000000000000066a493be1ea69d2b335152719acd54d735"),
        ),
        (
            Witness(78),
            hex_to_field("0x000000000000000000000000000000000027e49262bd388ce2d0f193988f3b8f"),
        ),
        (
            Witness(79),
            hex_to_field("0x000000000000000000000000000000dd783bff1a1cfc999bb29859cfb16c46fc"),
        ),
        (
            Witness(80),
            hex_to_field("0x000000000000000000000000000000000002c397073c8abce6d4140c9b961209"),
        ),
        (
            Witness(81),
            hex_to_field("0x000000000000000000000000000000750599be670db593af86e1923fe8a1bb18"),
        ),
        (
            Witness(82),
            hex_to_field("0x00000000000000000000000000000000002b7bba2d1efffce0d033f596b4d030"),
        ),
        (
            Witness(83),
            hex_to_field("0x0000000000000000000000000000008ffb571a4b3cf83533f3f71b99a04f6e6b"),
        ),
        (
            Witness(84),
            hex_to_field("0x00000000000000000000000000000000002c71c58b66498f903b3bbbda3d05ce"),
        ),
        (
            Witness(85),
            hex_to_field("0x0000000000000000000000000000002afaefbcbd080c84dcea90b54f4e0a858f"),
        ),
        (
            Witness(86),
            hex_to_field("0x0000000000000000000000000000000000039dce37f94d1bbd97ccea32a224fe"),
        ),
        (
            Witness(87),
            hex_to_field("0x00000000000000000000000000000075783c73cfe56847d848fd93b63bf32083"),
        ),
        (
            Witness(88),
            hex_to_field("0x000000000000000000000000000000000027dc44977efe6b3746a290706f4f72"),
        ),
        (
            Witness(89),
            hex_to_field("0x000000000000000000000000000000de0cbf2edc8f085b16d73652b15eced8f5"),
        ),
        (
            Witness(90),
            hex_to_field("0x00000000000000000000000000000000000a5366266dd7b71a10b356030226a2"),
        ),
        (
            Witness(91),
            hex_to_field("0x00000000000000000000000000000000a7588ec4d6809c90bb451005a3de3077"),
        ),
        (
            Witness(92),
            hex_to_field("0x0000000000000000000000000000000000136097d79e1b0ae373255e8760c499"),
        ),
        (
            Witness(93),
            hex_to_field("0x000000000000000000000000000000f2595d77bdf72e4acdb0b0b43969860d98"),
        ),
        (
            Witness(94),
            hex_to_field("0x000000000000000000000000000000000013dd7515ccac4095302d204f06f0bf"),
        ),
        (
            Witness(95),
            hex_to_field("0x000000000000000000000000000000057fe211dad1b706e49a3b55920fac20ec"),
        ),
        (
            Witness(96),
            hex_to_field("0x000000000000000000000000000000000016ff3501369121d410b445929239ba"),
        ),
        (
            Witness(97),
            hex_to_field("0x000000000000000000000000000000eb8007673c1ed10b834a695adf0068522a"),
        ),
        (
            Witness(98),
            hex_to_field("0x00000000000000000000000000000000001e190987ebd9cf480f608b82134a00"),
        ),
        (
            Witness(99),
            hex_to_field("0x0000000000000000000000000000000944f94301aa6da3016a226de04de52f4c"),
        ),
        (
            Witness(100),
            hex_to_field("0x00000000000000000000000000000000001e44194e60f0ab4ee0f77adc50f422"),
        ),
        (
            Witness(101),
            hex_to_field("0x0000000000000000000000000000006c2c7bea37dfbd20be6bed19efd743397a"),
        ),
        (
            Witness(102),
            hex_to_field("0x00000000000000000000000000000000002a017d0d9f40d0aeb5c8152ffddec5"),
        ),
        (
            Witness(103),
            hex_to_field("0x0000000000000000000000000000007f43efe5631bf48c872c317bed3b8bf12b"),
        ),
        (
            Witness(104),
            hex_to_field("0x000000000000000000000000000000000027579be0883627093cf8bdec0b72e7"),
        ),
        (
            Witness(105),
            hex_to_field("0x000000000000000000000000000000cef6108b89e89b35679431d113f3be7dff"),
        ),
        (
            Witness(106),
            hex_to_field("0x00000000000000000000000000000000000ddb2d01ec88ed69144177a4af3850"),
        ),
        (
            Witness(107),
            hex_to_field("0x0000000000000000000000000000000083e7ab1f26781948b36d131759f7c8c9"),
        ),
        (
            Witness(108),
            hex_to_field("0x00000000000000000000000000000000000a7fe830f1cb7a5d49d71877dd226a"),
        ),
        (
            Witness(109),
            hex_to_field("0x0000000000000000000000000000001834ecd1ce1e8e80812bdd95f960a45e57"),
        ),
        (
            Witness(110),
            hex_to_field("0x00000000000000000000000000000000002db7a5185064e6501ef61e989895a0"),
        ),
        (
            Witness(111),
            hex_to_field("0x000000000000000000000000000000363f0c994e91cecad25835338edee2294f"),
        ),
        (
            Witness(112),
            hex_to_field("0x00000000000000000000000000000000002eea648c8732596b1314fe2a4d2f05"),
        ),
        (
            Witness(113),
            hex_to_field("0x000000000000000000000000000000b2671d2ae51d31c1210433c3972bb64578"),
        ),
        (
            Witness(114),
            hex_to_field("0x00000000000000000000000000000000000ab49886c2b94bd0bd3f6ed1dbbe2c"),
        ),
        (
            Witness(115),
            hex_to_field("0x000000000000000000000000000000000000000000000000000000000000000a"),
        ),
        (
            Witness(116),
            hex_to_field("0x0000000000000000000000000000005e77a294b0829c1233b25f34cbd1e36ca5"),
        ),
        (
            Witness(117),
            hex_to_field("0x00000000000000000000000000000000001efb564c6d131a2005503e7bc96dfd"),
        ),
        (
            Witness(118),
            hex_to_field("0x0000000000000000000000000000003a2960d64558302ab11263ac1d4e99c792"),
        ),
        (
            Witness(119),
            hex_to_field("0x000000000000000000000000000000000027934be1b834b8444d8974e4c1c9bb"),
        ),
        (
            Witness(120),
            hex_to_field("0x000000000000000000000000000000a5e281184b833e3567ce8e285c80bd7dfc"),
        ),
        (
            Witness(121),
            hex_to_field("0x00000000000000000000000000000000002ef660bd670bea9dc8e18192cb71fa"),
        ),
        (
            Witness(122),
            hex_to_field("0x00000000000000000000000000000075b29302806ec08bb2c7af1b5463fc34fa"),
        ),
        (
            Witness(123),
            hex_to_field("0x00000000000000000000000000000000001138c220233f7b40034a4f49a23ae6"),
        ),
        (
            Witness(124),
            hex_to_field("0x000000000000000000000000000000c24fb0b91d6ea29b55a925f221c5b285d8"),
        ),
        (
            Witness(125),
            hex_to_field("0x000000000000000000000000000000000013ff3e12b86654ca896bfd6bbedd69"),
        ),
        (
            Witness(126),
            hex_to_field("0x0000000000000000000000000000005709282fede94015f85bce4c39d859e34a"),
        ),
        (
            Witness(127),
            hex_to_field("0x00000000000000000000000000000000000fb8a86b7540bfdc1c2784d7943400"),
        ),
        (
            Witness(128),
            hex_to_field("0x00000000000000000000000000000020bf9ff7ac6ddadf43c1f9128f13f66481"),
        ),
        (
            Witness(129),
            hex_to_field("0x000000000000000000000000000000000012f42d353e8a008c1c65650aea9720"),
        ),
        (
            Witness(130),
            hex_to_field("0x0000000000000000000000000000009b8c079fcd0a17aecbda82b255ac26131b"),
        ),
        (
            Witness(131),
            hex_to_field("0x000000000000000000000000000000000027fe6ea46f3898befbae77137e493e"),
        ),
        (
            Witness(132),
            hex_to_field("0x0000000000000000000000000000002a66a58be32207d7ac2e318e6d3235edac"),
        ),
        (
            Witness(133),
            hex_to_field("0x00000000000000000000000000000000000fa3dfdf2bbf7c51f39b861dc44be6"),
        ),
        (
            Witness(134),
            hex_to_field("0x0000000000000000000000000000003746eb9ded01fcafcc65c5d87f49141ee5"),
        ),
        (
            Witness(135),
            hex_to_field("0x00000000000000000000000000000000001e65f8c6b1af063d4103022b38cd3e"),
        ),
        (
            Witness(136),
            hex_to_field("0x00000000000000000000000000000046c520b61b4608d1bc2c98ca800765ebd7"),
        ),
        (
            Witness(137),
            hex_to_field("0x000000000000000000000000000000000020434f43987d0f71d0a1aa2ed8f270"),
        ),
        (
            Witness(138),
            hex_to_field("0x000000000000000000000000000000827b6b7c3b2a9c71a45a253a2a298c47f4"),
        ),
        (
            Witness(139),
            hex_to_field("0x000000000000000000000000000000000009e45e0d42b0e22cbde0f4667e6288"),
        ),
        (
            Witness(140),
            hex_to_field("0x000000000000000000000000000000c8150ed84dd7b794ce5427fe99040bcd3d"),
        ),
        (
            Witness(141),
            hex_to_field("0x00000000000000000000000000000000002696a5d48bf45b5a80619ef91013d4"),
        ),
        (
            Witness(142),
            hex_to_field("0x0000000000000000000000000000003a1caa16acc8da5032b2e836770312009d"),
        ),
        (
            Witness(143),
            hex_to_field("0x0000000000000000000000000000000000237a8423952c1c64e1e7c75da9d7cf"),
        ),
        (
            Witness(144),
            hex_to_field("0x0000000000000000000000000000000d8eb5fa6490a4cd67943b646d05bd0859"),
        ),
        (
            Witness(145),
            hex_to_field("0x0000000000000000000000000000000000159ebdb4a5c764c0346287984ed47d"),
        ),
        (
            Witness(146),
            hex_to_field("0x000000000000000000000000000000e862c821c535a49e93959d08dc9f2645b5"),
        ),
        (
            Witness(147),
            hex_to_field("0x00000000000000000000000000000000000c440edae454a8865dc27c8de51090"),
        ),
        (
            Witness(148),
            hex_to_field("0x000000000000000000000000000000a6973dd133a0e974b564e76d185a4b06b0"),
        ),
        (
            Witness(149),
            hex_to_field("0x000000000000000000000000000000000016248ed7566da68af6f2bc248763b4"),
        ),
        (
            Witness(150),
            hex_to_field("0x000000000000000000000000000000a568fd8430c974e995915c9265ac74617d"),
        ),
        (
            Witness(151),
            hex_to_field("0x000000000000000000000000000000000006e205349a7913be4af0af8778a0fd"),
        ),
        (
            Witness(152),
            hex_to_field("0x00000000000000000000000000000009fd63b6ca1767490d4ce191e7332fbdd6"),
        ),
        (
            Witness(153),
            hex_to_field("0x00000000000000000000000000000000000f95d28c7e720dc455fd46a532731e"),
        ),
        (
            Witness(154),
            hex_to_field("0x00000000000000000000000000000008d1b9d51b2425ddf4a15bc5307ea911b4"),
        ),
        (
            Witness(155),
            hex_to_field("0x000000000000000000000000000000000001131845742cefc926b7d2b7dc4b9c"),
        ),
        (
            Witness(156),
            hex_to_field("0x0000000000000000000000000000008dbc181365f1a3db87a66d527ca9d81ca5"),
        ),
        (
            Witness(157),
            hex_to_field("0x00000000000000000000000000000000000a6f78cdcd1e2177580e6c89c23235"),
        ),
        (
            Witness(158),
            hex_to_field("0x0000000000000000000000000000004723acbe295108f00ff760c0671d2d4bbf"),
        ),
        (
            Witness(159),
            hex_to_field("0x000000000000000000000000000000000006058d93abb1d596501ee4c3f62971"),
        ),
        (
            Witness(160),
            hex_to_field("0x08bacf9fdaba383e584559b8cd64ae8c04e670d9203f90c6b49efac7f00f5003"),
        ),
        (
            Witness(161),
            hex_to_field("0x18541473055ebbcaefe15759125b820ed1c6b932af2659c5280bdf70bd5c09cc"),
        ),
        (
            Witness(162),
            hex_to_field("0x161e0a0cb1aa6028cabb8ccb98646a9b0976618cad99bb1145c4d25cecef50be"),
        ),
        (
            Witness(163),
            hex_to_field("0x0d353ffc0833fd6e1947133f5391544ed7dde0fbfa0109ec7a54baafb117b1ca"),
        ),
        (
            Witness(164),
            hex_to_field("0x1a5209fd1dcf2705b7081b4e3bf7b2c33dd00ac4b2becfdf8ee7927703ea0357"),
        ),
        (
            Witness(165),
            hex_to_field("0x1d247635110c48df6f62387026c5823f0eb9d843848fe7b8e1a9a96b1c6ad763"),
        ),
        (
            Witness(166),
            hex_to_field("0x1cc4a7a8be5edc32432191b0ee2a9051d3b6384313c6b9e5efe8cd8712c872f2"),
        ),
        (
            Witness(167),
            hex_to_field("0x2c8b6fa617041faeb2e814b39c288ff607ac03d746f3c0e622860720dfb24b83"),
        ),
        (
            Witness(168),
            hex_to_field("0x1ecc99a77fda5d79a6426b18049876b36ad1a1aba693518b1b976360630c2f55"),
        ),
        (
            Witness(169),
            hex_to_field("0x2f75dc15bb6fdd3d9762fe74485c5ead7a5476c11cd44ed9f43324028cd2dd68"),
        ),
        (
            Witness(170),
            hex_to_field("0x0e20add7931c78604ef7986fe7b286ab582842a23b4c09e8ec03d8d88a31969c"),
        ),
        (
            Witness(171),
            hex_to_field("0x2467bb747466b69b6b4deeaac4a82e32ca7585194cd838912a65d12f912b5c6c"),
        ),
        (
            Witness(172),
            hex_to_field("0x23edab06b87cf9fd4a5f0161287283d97a9bcdbdd68779e08cad3e763420bd20"),
        ),
        (
            Witness(173),
            hex_to_field("0x2817c054ad1ac5454f58ff525196ed920ba54fbb4d86820a5a414aaa61d7d1b1"),
        ),
        (
            Witness(174),
            hex_to_field("0x12d63d1f6ed0a080694a209534ee08e4758b0382f9bab7e13aafcbcb62ecc8d0"),
        ),
        (
            Witness(175),
            hex_to_field("0x153104c35caab490767364a7db8bca01043c63f358f20edd6205c544cf4a61ea"),
        ),
        (
            Witness(176),
            hex_to_field("0x178bcc674a84c8a0839ca8ba82298b1d92edc463b82965d9895bbebe3ba7fb04"),
        ),
        (
            Witness(177),
            hex_to_field("0x1224834d4b8a36290e11b8b153d81062ba503c36d6e7ef41916b647517a6e632"),
        ),
        (
            Witness(178),
            hex_to_field("0x13112373ea4e5bf7e041a7312167b4f82653ead2f5e5e3d4d07bafd79ca690b6"),
        ),
        (
            Witness(179),
            hex_to_field("0x26b7669e3463c6d162363b2cd0e8f6720aa97f9cdb04a8340fce7ead2421af56"),
        ),
        (
            Witness(180),
            hex_to_field("0x120d09593529a665d992bf009fc6268a9088c95f401784f939d5ed1649a4e779"),
        ),
        (
            Witness(181),
            hex_to_field("0x1c415baf2638f0c09def30dfcf650d56b0508544769813d1d807b1b114632d38"),
        ),
        (
            Witness(182),
            hex_to_field("0x1e9c2353141304d0ab1874f27602ce733f01e5b4d5cf6acdff5dab2a80c0c652"),
        ),
        (
            Witness(183),
            hex_to_field("0x20f6eaf701ed18e0b841b9051ca08f8fcdb346253506c1ca26b3a4a3ed1e5f6c"),
        ),
        (
            Witness(184),
            hex_to_field("0x2351b29aefc72cf0c56afd17c33e50ac5c64a695943e18c64e099e1d597bf886"),
        ),
        (
            Witness(185),
            hex_to_field("0x19e6940b385edcb090c5eccd28c74c3a219f24d41760bcd5b0b1b837a805941e"),
        ),
        (
            Witness(186),
            hex_to_field("0x2cd7e4b967101d6ee0f2a33521762cace8ffe35930bc210554e8307df664c899"),
        ),
        (
            Witness(187),
            hex_to_field("0x041f06de46e4862d5d59c363c119a79629261d6aa18aa737c288ac7f4bfb4153"),
        ),
        (
            Witness(188),
            hex_to_field("0x2dc39620da58c2822418179ba6f61de6d31ee938c79a5ca15c473aef7ca1e824"),
        ),
        (
            Witness(189),
            hex_to_field("0x00000000000000000000000000000000ffbd168649f4e00f0baef4ec3a08615f"),
        ),
        (
            Witness(190),
            hex_to_field("0x18fbeff26a87cb38f373584bbd02d016fed78aefc6462811a23006679509b3a9"),
        ),
        (
            Witness(191),
            hex_to_field("0x1888e78ad37d146406e710ae2dbd244877263b133875d090f7615a1e9c0ac083"),
        ),
        (
            Witness(192),
            hex_to_field("0x2196fbe28ce9ce0e0e202bbf1268cabdcd0a2c03588e118765ba1ee1a16f2dc7"),
        ),
        (
            Witness(193),
            hex_to_field("0x0137bc731354b1531dbdcbfc83802605035f69f937f9a7311a57e6d7126368ba"),
        ),
        (
            Witness(194),
            hex_to_field("0x19f38da8f0717fe78812addd655ef59411805d70eb731d5da309ad111698e8d0"),
        ),
        (
            Witness(195),
            hex_to_field("0x155452e2824d5bd4fd8f8e5feaa4bd7abe783613d6b78cf88377a48e9f7e70c2"),
        ),
        (
            Witness(196),
            hex_to_field("0x2396966b07a6e535a9ae30883a97e854ff2425c6dcfa34bda164394ba919191f"),
        ),
        (
            Witness(197),
            hex_to_field("0x09374f47b862065ac0ac49ceb02b5cc0d925af1980ab2bd5f4d9df555e8c4c91"),
        ),
        (
            Witness(198),
            hex_to_field("0x26366e50b5c7244ffc3ecdf50a65180742b1c53092659bd1db852bdd726d52f3"),
        ),
        (
            Witness(199),
            hex_to_field("0x12d13ee6d1faa21b7f810c64e31d7af08409f2ff2a669b3c7e4e82d1964e5954"),
        ),
        (
            Witness(200),
            hex_to_field("0x2fd05defcf5fc010bb13908b3d573636ed9609163c210b3864f9cf59aa2f5fb6"),
        ),
        (
            Witness(201),
            hex_to_field("0x00000000000000000000000000000046955fdfd58ca9013b39025ae688416131"),
        ),
        (
            Witness(202),
            hex_to_field("0x00000000000000000000000000000000001d335d2fb9857cbc49e72cf34e86a5"),
        ),
        (
            Witness(203),
            hex_to_field("0x0000000000000000000000000000000c6a8930092b36c72dbd0a7f4b65533c19"),
        ),
        (
            Witness(204),
            hex_to_field("0x00000000000000000000000000000000000d099ff72ffae0f73756528d629a5e"),
        ),
        (
            Witness(205),
            hex_to_field("0x0000000000000000000000000000008c8d80c3f2886519cb37a563f88f166cb8"),
        ),
        (
            Witness(206),
            hex_to_field("0x00000000000000000000000000000000000393e9f6fdc31492e4b3da33fa5fe4"),
        ),
        (
            Witness(207),
            hex_to_field("0x000000000000000000000000000000417fb818a6933554bf3ff602f1f450728d"),
        ),
        (
            Witness(208),
            hex_to_field("0x00000000000000000000000000000000002074eb75888a752047676f72f5343f"),
        ),
        (
            Witness(209),
            hex_to_field("0x000000000000000000000000000000000000000000000000000000000000000a"),
        ),
        (
            Witness(210),
            hex_to_field("0x284158f92a5305f662f78fc36a397fb8eb44d229fd22152e2dc085cad142c3c2"),
        ),
        (
            Witness(211),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(212),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(213),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(214),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(215),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(216),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(217),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(218),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(219),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(220),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(221),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(222),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(223),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(224),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(225),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
        (
            Witness(226),
            hex_to_field("0x0000000000000000000000000000000000000000000000000000000000000000"),
        ),
    ])
    .into();

    let mut acvm = ACVM::new(&StubbedBackend, circuit.opcodes, witness_assignments);
    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve();
    match solver_status {
        ACVMStatus::Solved => todo!("solved"),
        ACVMStatus::InProgress => todo!("in prog"),
        ACVMStatus::Failure(_) => todo!("fail"),
        ACVMStatus::RequiresForeignCall(_) => todo!("ffi"),
    }
}

#[test]
#[ignore]
fn inversion_brillig_oracle_equivalence() {
    // Opcodes below describe the following:
    // fn main(x : Field, y : pub Field) {
    //     let z = x + y;
    //     assert( 1/z == Oracle("inverse", x + y) );
    // }
    // Also performs an unrelated equality check
    // just for the sake of testing multiple brillig opcodes.
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_oracle = Witness(3);
    let w_z = Witness(4);
    let w_z_inverse = Witness(5);
    let w_x_plus_y = Witness(6);
    let w_equal_res = Witness(7);

    let equal_opcode = BrilligOpcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: RegisterIndex::from(0),
        rhs: RegisterIndex::from(1),
        destination: RegisterIndex::from(2),
    };

    let brillig_data = Brillig {
        inputs: vec![
            BrilligInputs::Single(Expression {
                // Input Register 0
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                q_c: fe_0,
            }),
            BrilligInputs::Single(Expression::default()), // Input Register 1
        ],
        // This tells the BrilligSolver which witnesses its output registers correspond to
        outputs: vec![
            BrilligOutputs::Simple(w_x_plus_y), // Output Register 0 - from input
            BrilligOutputs::Simple(w_oracle),   // Output Register 1
            BrilligOutputs::Simple(w_equal_res), // Output Register 2
        ],
        // stack of foreign call/oracle resolutions, starts empty
        foreign_call_results: vec![],
        bytecode: vec![
            equal_opcode,
            // Oracles are named 'foreign calls' in brillig
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(1))],
                inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
            },
        ],
        predicate: None,
    };

    let opcodes = vec![
        Opcode::Brillig(brillig_data),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
            q_c: fe_0,
        }),
        // Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![(fe_1, w_z, w_z_inverse)],
            linear_combinations: vec![],
            q_c: -fe_1,
        }),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(-fe_1, w_oracle), (fe_1, w_z_inverse)],
            q_c: fe_0,
        }),
    ];

    let witness_assignments = BTreeMap::from([
        (Witness(1), FieldElement::from(2u128)),
        (Witness(2), FieldElement::from(3u128)),
    ])
    .into();

    let mut acvm = ACVM::new(&StubbedBackend, opcodes, witness_assignments);
    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve();

    assert!(
        matches!(solver_status, ACVMStatus::RequiresForeignCall(_)),
        "should require foreign call response"
    );
    assert_eq!(acvm.instruction_pointer(), 0, "brillig should have been removed");

    let foreign_call_wait_info: &ForeignCallWaitInfo =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    // As caller of VM, need to resolve foreign calls
    let foreign_call_result = Value::from(foreign_call_wait_info.inputs[0][0].to_field().inverse());
    // Alter Brillig oracle opcode with foreign call resolution
    acvm.resolve_pending_foreign_call(foreign_call_result.into());

    // After filling data request, continue solving
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");

    // ACVM should be able to be finalized in `Solved` state.
    acvm.finalize();
}

#[test]
#[ignore]
fn double_inversion_brillig_oracle() {
    // Opcodes below describe the following:
    // fn main(x : Field, y : pub Field) {
    //     let z = x + y;
    //     let ij = i + j;
    //     assert( 1/z == Oracle("inverse", x + y) );
    //     assert( 1/ij == Oracle("inverse", i + j) );
    // }
    // Also performs an unrelated equality check
    // just for the sake of testing multiple brillig opcodes.
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_oracle = Witness(3);
    let w_z = Witness(4);
    let w_z_inverse = Witness(5);
    let w_x_plus_y = Witness(6);
    let w_equal_res = Witness(7);
    let w_i = Witness(8);
    let w_j = Witness(9);
    let w_ij_oracle = Witness(10);
    let w_i_plus_j = Witness(11);

    let equal_opcode = BrilligOpcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: RegisterIndex::from(0),
        rhs: RegisterIndex::from(1),
        destination: RegisterIndex::from(4),
    };

    let brillig_data = Brillig {
        inputs: vec![
            BrilligInputs::Single(Expression {
                // Input Register 0
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                q_c: fe_0,
            }),
            BrilligInputs::Single(Expression::default()), // Input Register 1
            BrilligInputs::Single(Expression {
                // Input Register 2
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_i), (fe_1, w_j)],
                q_c: fe_0,
            }),
        ],
        outputs: vec![
            BrilligOutputs::Simple(w_x_plus_y), // Output Register 0 - from input
            BrilligOutputs::Simple(w_oracle),   // Output Register 1
            BrilligOutputs::Simple(w_i_plus_j), // Output Register 2 - from input
            BrilligOutputs::Simple(w_ij_oracle), // Output Register 3
            BrilligOutputs::Simple(w_equal_res), // Output Register 4
        ],
        // stack of foreign call/oracle resolutions, starts empty
        foreign_call_results: vec![],
        bytecode: vec![
            equal_opcode,
            // Oracles are named 'foreign calls' in brillig
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(1))],
                inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
            },
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(3))],
                inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(2))],
            },
        ],
        predicate: None,
    };

    let opcodes = vec![
        Opcode::Brillig(brillig_data),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
            q_c: fe_0,
        }),
        // Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![(fe_1, w_z, w_z_inverse)],
            linear_combinations: vec![],
            q_c: -fe_1,
        }),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(-fe_1, w_oracle), (fe_1, w_z_inverse)],
            q_c: fe_0,
        }),
    ];

    let witness_assignments = BTreeMap::from([
        (Witness(1), FieldElement::from(2u128)),
        (Witness(2), FieldElement::from(3u128)),
        (Witness(8), FieldElement::from(5u128)),
        (Witness(9), FieldElement::from(10u128)),
    ])
    .into();

    let mut acvm = ACVM::new(&StubbedBackend, opcodes, witness_assignments);

    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve();
    assert!(
        matches!(solver_status, ACVMStatus::RequiresForeignCall(_)),
        "should require foreign call response"
    );
    assert_eq!(acvm.instruction_pointer(), 0, "should stall on brillig");

    let foreign_call_wait_info: &ForeignCallWaitInfo =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    let x_plus_y_inverse = Value::from(foreign_call_wait_info.inputs[0][0].to_field().inverse());

    // Resolve Brillig foreign call
    acvm.resolve_pending_foreign_call(x_plus_y_inverse.into());

    // After filling data request, continue solving
    let solver_status = acvm.solve();
    assert!(
        matches!(solver_status, ACVMStatus::RequiresForeignCall(_)),
        "should require foreign call response"
    );
    assert_eq!(acvm.instruction_pointer(), 0, "should stall on brillig");

    let foreign_call_wait_info =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    let i_plus_j_inverse = Value::from(foreign_call_wait_info.inputs[0][0].to_field().inverse());
    assert_ne!(x_plus_y_inverse, i_plus_j_inverse);

    // Alter Brillig oracle opcode
    acvm.resolve_pending_foreign_call(i_plus_j_inverse.into());

    // After filling data request, continue solving
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");

    // ACVM should be able to be finalized in `Solved` state.
    acvm.finalize();
}

#[test]
fn oracle_dependent_execution() {
    // This test ensures that we properly track the list of opcodes which still need to be resolved
    // across any brillig foreign calls we may have to perform.
    //
    // Opcodes below describe the following:
    // fn main(x : Field, y : pub Field) {
    //     assert(x == y);
    //     let x_inv = Oracle("inverse", x);
    //     let y_inv = Oracle("inverse", y);
    //
    //     assert(x_inv == y_inv);
    // }
    // Also performs an unrelated equality check
    // just for the sake of testing multiple brillig opcodes.
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_x_inv = Witness(3);
    let w_y_inv = Witness(4);

    let brillig_data = Brillig {
        inputs: vec![
            BrilligInputs::Single(w_x.into()),            // Input Register 0
            BrilligInputs::Single(Expression::default()), // Input Register 1
            BrilligInputs::Single(w_y.into()),            // Input Register 2,
        ],
        outputs: vec![
            BrilligOutputs::Simple(w_x),     // Output Register 0 - from input
            BrilligOutputs::Simple(w_y_inv), // Output Register 1
            BrilligOutputs::Simple(w_y),     // Output Register 2 - from input
            BrilligOutputs::Simple(w_y_inv), // Output Register 3
        ],
        // stack of foreign call/oracle resolutions, starts empty
        foreign_call_results: vec![],
        bytecode: vec![
            // Oracles are named 'foreign calls' in brillig
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(1))],
                inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
            },
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(3))],
                inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(2))],
            },
        ],
        predicate: None,
    };

    // This equality check can be executed immediately before resolving any foreign calls.
    let equality_check = Expression {
        mul_terms: vec![],
        linear_combinations: vec![(-fe_1, w_x), (fe_1, w_y)],
        q_c: fe_0,
    };

    // This equality check relies on the outputs of the Brillig call.
    // It then cannot be solved until the foreign calls are resolved.
    let inverse_equality_check = Expression {
        mul_terms: vec![],
        linear_combinations: vec![(-fe_1, w_x_inv), (fe_1, w_y_inv)],
        q_c: fe_0,
    };

    let opcodes = vec![
        Opcode::Arithmetic(equality_check),
        Opcode::Brillig(brillig_data),
        Opcode::Arithmetic(inverse_equality_check),
    ];

    let witness_assignments =
        BTreeMap::from([(w_x, FieldElement::from(2u128)), (w_y, FieldElement::from(2u128))]).into();

    let mut acvm = ACVM::new(&StubbedBackend, opcodes, witness_assignments);

    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve();
    assert!(
        matches!(solver_status, ACVMStatus::RequiresForeignCall(_)),
        "should require foreign call response"
    );
    assert_eq!(acvm.instruction_pointer(), 1, "should stall on brillig");

    let foreign_call_wait_info: &ForeignCallWaitInfo =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    // Resolve Brillig foreign call
    let x_inverse = Value::from(foreign_call_wait_info.inputs[0][0].to_field().inverse());
    acvm.resolve_pending_foreign_call(x_inverse.into());

    // After filling data request, continue solving
    let solver_status = acvm.solve();
    assert!(
        matches!(solver_status, ACVMStatus::RequiresForeignCall(_)),
        "should require foreign call response"
    );
    assert_eq!(acvm.instruction_pointer(), 1, "should stall on brillig");

    let foreign_call_wait_info: &ForeignCallWaitInfo =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    // Resolve Brillig foreign call
    let y_inverse = Value::from(foreign_call_wait_info.inputs[0][0].to_field().inverse());
    acvm.resolve_pending_foreign_call(y_inverse.into());

    // We've resolved all the brillig foreign calls so we should be able to complete execution now.

    // After filling data request, continue solving
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");

    // ACVM should be able to be finalized in `Solved` state.
    acvm.finalize();
}

#[test]
fn brillig_oracle_predicate() {
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_oracle = Witness(3);
    let w_x_plus_y = Witness(4);
    let w_equal_res = Witness(5);
    let w_lt_res = Witness(6);

    let equal_opcode = BrilligOpcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: RegisterIndex::from(0),
        rhs: RegisterIndex::from(1),
        destination: RegisterIndex::from(2),
    };

    let brillig_opcode = Opcode::Brillig(Brillig {
        inputs: vec![
            BrilligInputs::Single(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                q_c: fe_0,
            }),
            BrilligInputs::Single(Expression::default()),
        ],
        outputs: vec![
            BrilligOutputs::Simple(w_x_plus_y),
            BrilligOutputs::Simple(w_oracle),
            BrilligOutputs::Simple(w_equal_res),
            BrilligOutputs::Simple(w_lt_res),
        ],
        bytecode: vec![
            equal_opcode,
            // Oracles are named 'foreign calls' in brillig
            BrilligOpcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(1))],
                inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
            },
        ],
        predicate: Some(Expression::default()),
        // oracle results
        foreign_call_results: vec![],
    });

    let opcodes = vec![brillig_opcode];

    let witness_assignments = BTreeMap::from([
        (Witness(1), FieldElement::from(2u128)),
        (Witness(2), FieldElement::from(3u128)),
    ])
    .into();

    let mut acvm = ACVM::new(&StubbedBackend, opcodes, witness_assignments);
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");

    // ACVM should be able to be finalized in `Solved` state.
    acvm.finalize();
}
#[test]
fn unsatisfied_opcode_resolved() {
    let a = Witness(0);
    let b = Witness(1);
    let c = Witness(2);
    let d = Witness(3);

    // a = b + c + d;
    let opcode_a = Expression {
        mul_terms: vec![],
        linear_combinations: vec![
            (FieldElement::one(), a),
            (-FieldElement::one(), b),
            (-FieldElement::one(), c),
            (-FieldElement::one(), d),
        ],
        q_c: FieldElement::zero(),
    };

    let mut values = WitnessMap::new();
    values.insert(a, FieldElement::from(4_i128));
    values.insert(b, FieldElement::from(2_i128));
    values.insert(c, FieldElement::from(1_i128));
    values.insert(d, FieldElement::from(2_i128));

    let opcodes = vec![Opcode::Arithmetic(opcode_a)];
    let mut acvm = ACVM::new(&StubbedBackend, opcodes, values);
    let solver_status = acvm.solve();
    assert_eq!(
        solver_status,
        ACVMStatus::Failure(OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: ErrorLocation::Resolved(OpcodeLocation::Acir(0)),
        }),
        "The first opcode is not satisfiable, expected an error indicating this"
    );
}

#[test]
fn unsatisfied_opcode_resolved_brillig() {
    let a = Witness(0);
    let b = Witness(1);
    let c = Witness(2);
    let d = Witness(3);

    let fe_1 = FieldElement::one();
    let fe_0 = FieldElement::zero();
    let w_x = Witness(4);
    let w_y = Witness(5);
    let w_result = Witness(6);

    let equal_opcode = BrilligOpcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: RegisterIndex::from(0),
        rhs: RegisterIndex::from(1),
        destination: RegisterIndex::from(2),
    };
    // Jump pass the trap if the values are equal, else
    // jump to the trap
    let location_of_stop = 3;

    let jmp_if_opcode =
        BrilligOpcode::JumpIf { condition: RegisterIndex::from(2), location: location_of_stop };

    let trap_opcode = BrilligOpcode::Trap;
    let stop_opcode = BrilligOpcode::Stop;

    let brillig_opcode = Opcode::Brillig(Brillig {
        inputs: vec![
            BrilligInputs::Single(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x)],
                q_c: fe_0,
            }),
            BrilligInputs::Single(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_y)],
                q_c: fe_0,
            }),
        ],
        outputs: vec![BrilligOutputs::Simple(w_result)],
        bytecode: vec![equal_opcode, jmp_if_opcode, trap_opcode, stop_opcode],
        predicate: Some(Expression::one()),
        // oracle results
        foreign_call_results: vec![],
    });

    let opcode_a = Expression {
        mul_terms: vec![],
        linear_combinations: vec![
            (FieldElement::one(), a),
            (-FieldElement::one(), b),
            (-FieldElement::one(), c),
            (-FieldElement::one(), d),
        ],
        q_c: FieldElement::zero(),
    };

    let mut values = WitnessMap::new();
    values.insert(a, FieldElement::from(4_i128));
    values.insert(b, FieldElement::from(2_i128));
    values.insert(c, FieldElement::from(1_i128));
    values.insert(d, FieldElement::from(2_i128));
    values.insert(w_x, FieldElement::from(0_i128));
    values.insert(w_y, FieldElement::from(1_i128));
    values.insert(w_result, FieldElement::from(0_i128));

    let opcodes = vec![brillig_opcode, Opcode::Arithmetic(opcode_a)];

    let mut acvm = ACVM::new(&StubbedBackend, opcodes, values);
    let solver_status = acvm.solve();
    assert_eq!(
        solver_status,
        ACVMStatus::Failure(OpcodeResolutionError::BrilligFunctionFailed {
            message: "explicit trap hit in brillig".to_string(),
            call_stack: vec![OpcodeLocation::Brillig { acir_index: 0, brillig_index: 2 }]
        }),
        "The first opcode is not satisfiable, expected an error indicating this"
    );
}

#[test]
fn memory_operations() {
    let initial_witness = WitnessMap::from(BTreeMap::from_iter([
        (Witness(1), FieldElement::from(1u128)),
        (Witness(2), FieldElement::from(2u128)),
        (Witness(3), FieldElement::from(3u128)),
        (Witness(4), FieldElement::from(4u128)),
        (Witness(5), FieldElement::from(5u128)),
        (Witness(6), FieldElement::from(4u128)),
    ]));

    let block_id = BlockId(0);

    let init = Opcode::MemoryInit { block_id, init: (1..6).map(Witness).collect() };

    let read_op = Opcode::MemoryOp {
        block_id,
        op: MemOp::read_at_mem_index(Witness(6).into(), Witness(7)),
        predicate: None,
    };

    let expression = Opcode::Arithmetic(Expression {
        mul_terms: Vec::new(),
        linear_combinations: vec![
            (FieldElement::one(), Witness(7)),
            (-FieldElement::one(), Witness(8)),
        ],
        q_c: FieldElement::one(),
    });

    let opcodes = vec![init, read_op, expression];

    let mut acvm = ACVM::new(&StubbedBackend, opcodes, initial_witness);
    let solver_status = acvm.solve();
    assert_eq!(solver_status, ACVMStatus::Solved);
    let witness_map = acvm.finalize();

    assert_eq!(witness_map[&Witness(8)], FieldElement::from(6u128));
}
