/// aarch64 documentation
///
/// ## Features
/// | Feature | Description | Also Enables<sup>†</sup> |
/// | ------- | ----------- | ------------------------ |
/// | `aes` | Enable AES support (FEAT_AES, FEAT_PMULL). | `neon` |
/// | `bf16` | Enable BFloat16 Extension (FEAT_BF16). |  |
/// | `bti` | Enable Branch Target Identification (FEAT_BTI). |  |
/// | `crc` | Enable ARMv8 CRC-32 checksum instructions (FEAT_CRC32). |  |
/// | `dit` | Enable v8.4-A Data Independent Timing instructions (FEAT_DIT). |  |
/// | `dotprod` | Enable dot product support (FEAT_DotProd). |  |
/// | `dpb` | Enable v8.2 data Cache Clean to Point of Persistence (FEAT_DPB). |  |
/// | `dpb2` | Enable v8.5 Cache Clean to Point of Deep Persistence (FEAT_DPB2). |  |
/// | `f32mm` | Enable Matrix Multiply FP32 Extension (FEAT_F32MM). | `fp16`, `neon`, `sve` |
/// | `f64mm` | Enable Matrix Multiply FP64 Extension (FEAT_F64MM). | `fp16`, `neon`, `sve` |
/// | `fcma` | Enable v8.3-A Floating-point complex number support (FEAT_FCMA). | `neon` |
/// | `fhm` | Enable FP16 FML instructions (FEAT_FHM). | `fp16`, `neon` |
/// | `flagm` | Enable v8.4-A Flag Manipulation Instructions (FEAT_FlagM). |  |
/// | `fp16` | Full FP16 (FEAT_FP16). | `neon` |
/// | `frintts` | Enable FRInt[32|64][Z|X] instructions that round a floating-point number to an integer (in FP format) forcing it to fit into a 32- or 64-bit int (FEAT_FRINTTS). |  |
/// | `i8mm` | Enable Matrix Multiply Int8 Extension (FEAT_I8MM). |  |
/// | `jsconv` | Enable v8.3-A JavaScript FP conversion instructions (FEAT_JSCVT). | `neon` |
/// | `lor` | Enables ARM v8.1 Limited Ordering Regions extension (FEAT_LOR). |  |
/// | `lse` | Enable ARMv8.1 Large System Extension (LSE) atomic instructions (FEAT_LSE). |  |
/// | `mte` | Enable Memory Tagging Extension (FEAT_MTE, FEAT_MTE2). |  |
/// | `neon` | Enable Advanced SIMD instructions (FEAT_AdvSIMD). |  |
/// | `paca` | Enable v8.3-A Pointer Authentication extension (FEAT_PAuth). | `pacg` |
/// | `pacg` | Enable v8.3-A Pointer Authentication extension (FEAT_PAuth). | `paca` |
/// | `pan` | Enables ARM v8.1 Privileged Access-Never extension (FEAT_PAN). |  |
/// | `pmuv3` | Enable Code Generation for ARMv8 PMUv3 Performance Monitors extension (FEAT_PMUv3). |  |
/// | `rand` | Enable Random Number generation instructions (FEAT_RNG). |  |
/// | `ras` | Enable ARMv8 Reliability, Availability and Serviceability Extensions (FEAT_RAS, FEAT_RASv1p1). |  |
/// | `rcpc` | Enable support for RCPC extension (FEAT_LRCPC). |  |
/// | `rcpc2` | Enable v8.4-A RCPC instructions with Immediate Offsets (FEAT_LRCPC2). | `rcpc` |
/// | `rdm` | Enable ARMv8.1 Rounding Double Multiply Add/Subtract instructions (FEAT_RDM). |  |
/// | `sb` | Enable v8.5 Speculation Barrier (FEAT_SB). |  |
/// | `sha2` | Enable SHA1 and SHA256 support (FEAT_SHA1, FEAT_SHA256). | `neon` |
/// | `sha3` | Enable SHA512 and SHA3 support (FEAT_SHA3, FEAT_SHA512). | `neon`, `sha2` |
/// | `sm4` | Enable SM3 and SM4 support (FEAT_SM4, FEAT_SM3). | `neon` |
/// | `spe` | Enable Statistical Profiling extension (FEAT_SPE). |  |
/// | `ssbs` | Enable Speculative Store Bypass Safe bit (FEAT_SSBS, FEAT_SSBS2). |  |
/// | `sve` | Enable Scalable Vector Extension (SVE) instructions (FEAT_SVE). | `fp16`, `neon` |
/// | `sve2` | Enable Scalable Vector Extension 2 (SVE2) instructions (FEAT_SVE2). | `fp16`, `neon`, `sve` |
/// | `sve2-aes` | Enable AES SVE2 instructions (FEAT_SVE_AES, FEAT_SVE_PMULL128). | `aes`, `fp16`, `neon`, `sve`, `sve2` |
/// | `sve2-bitperm` | Enable bit permutation SVE2 instructions (FEAT_SVE_BitPerm). | `fp16`, `neon`, `sve`, `sve2` |
/// | `sve2-sha3` | Enable SHA3 SVE2 instructions (FEAT_SVE_SHA3). | `fp16`, `neon`, `sha2`, `sha3`, `sve`, `sve2` |
/// | `sve2-sm4` | Enable SM4 SVE2 instructions (FEAT_SVE_SM4). | `fp16`, `neon`, `sm4`, `sve`, `sve2` |
/// | `tme` | Enable Transactional Memory Extension (FEAT_TME). |  |
/// | `v8.1a` | Support ARM v8.1a instructions. | `crc`, `lor`, `lse`, `pan`, `rdm`, `vh` |
/// | `v8.2a` | Support ARM v8.2a instructions. | `crc`, `dpb`, `lor`, `lse`, `pan`, `ras`, `rdm`, `v8.1a`, `vh` |
/// | `v8.3a` | Support ARM v8.3a instructions. | `crc`, `dpb`, `fcma`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `ras`, `rcpc`, `rdm`, `v8.1a`, `v8.2a`, `vh` |
/// | `v8.4a` | Support ARM v8.4a instructions. | `crc`, `dit`, `dotprod`, `dpb`, `fcma`, `flagm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `ras`, `rcpc`, `rcpc2`, `rdm`, `v8.1a`, `v8.2a`, `v8.3a`, `vh` |
/// | `v8.5a` | Support ARM v8.5a instructions. | `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `flagm`, `frintts`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `vh` |
/// | `v8.6a` | Support ARM v8.6a instructions. | `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `flagm`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `vh` |
/// | `v8.7a` | Support ARM v8.7a instructions. | `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `flagm`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `vh` |
/// | `vh` | Enables ARM v8.1 Virtual Host extension (FEAT_VHE). |  |
/// | `crt-static` | Enables C Run-time Libraries to be statically linked. |  |
///
/// <sup>†</sup> This is often empirical, rather than specified in any standard, i.e. all available CPUs with a particular feature also have another feature.
///
/// ## CPUs
/// | CPU | Enabled Features |
/// | --- | -------- |
/// | `a64fx` | `crc`, `dpb`, `fcma`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rdm`, `sha2`, `sve`, `v8.1a`, `v8.2a`, `vh` |
/// | `ampere1` | `aes`, `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `flagm`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `rand`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `sha2`, `sha3`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `vh` |
/// | `ampere1a` | `aes`, `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `flagm`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `mte`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `rand`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `sha2`, `sha3`, `sm4`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `vh` |
/// | `ampere1b` | `aes`, `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `mte`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `rand`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `sha2`, `sha3`, `sm4`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `v8.7a`, `vh` |
/// | `apple-a10` | `aes`, `crc`, `lor`, `neon`, `pan`, `pmuv3`, `rdm`, `sha2`, `vh` |
/// | `apple-a11` | `aes`, `crc`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rdm`, `sha2`, `v8.1a`, `v8.2a`, `vh` |
/// | `apple-a12` | `aes`, `crc`, `dpb`, `fcma`, `fp16`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `v8.1a`, `v8.2a`, `v8.3a`, `vh` |
/// | `apple-a13` | `aes`, `crc`, `dit`, `dotprod`, `dpb`, `fcma`, `fhm`, `flagm`, `fp16`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sha2`, `sha3`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `vh` |
/// | `apple-a14` | `aes`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `sha2`, `sha3`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `vh` |
/// | `apple-a15` | `aes`, `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `sha2`, `sha3`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `vh` |
/// | `apple-a16` | `aes`, `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `sha2`, `sha3`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `vh` |
/// | `apple-a17` | `aes`, `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `sha2`, `sha3`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `vh` |
/// | `apple-a7` | `aes`, `neon`, `pmuv3`, `sha2` |
/// | `apple-a8` | `aes`, `neon`, `pmuv3`, `sha2` |
/// | `apple-a9` | `aes`, `neon`, `pmuv3`, `sha2` |
/// | `apple-latest` | `aes`, `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `sha2`, `sha3`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `vh` |
/// | `apple-m1` | `aes`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `sha2`, `sha3`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `vh` |
/// | `apple-m2` | `aes`, `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `sha2`, `sha3`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `vh` |
/// | `apple-m3` | `aes`, `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `sha2`, `sha3`, `ssbs`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `vh` |
/// | `apple-s4` | `aes`, `crc`, `dpb`, `fcma`, `fp16`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `v8.1a`, `v8.2a`, `v8.3a`, `vh` |
/// | `apple-s5` | `aes`, `crc`, `dpb`, `fcma`, `fp16`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `v8.1a`, `v8.2a`, `v8.3a`, `vh` |
/// | `carmel` | `aes`, `crc`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `ras`, `rdm`, `sha2`, `v8.1a`, `v8.2a`, `vh` |
/// | `cortex-a34` | `aes`, `crc`, `neon`, `pmuv3`, `sha2` |
/// | `cortex-a35` | `aes`, `crc`, `neon`, `pmuv3`, `sha2` |
/// | `cortex-a510` | `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `mte`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `ssbs`, `sve`, `sve2`, `sve2-bitperm`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `vh` |
/// | `cortex-a520` | `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `mte`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `ssbs`, `sve`, `sve2`, `sve2-bitperm`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `v8.7a`, `vh` |
/// | `cortex-a53` | `aes`, `crc`, `neon`, `pmuv3`, `sha2` |
/// | `cortex-a55` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `v8.1a`, `v8.2a`, `vh` |
/// | `cortex-a57` | `aes`, `crc`, `neon`, `pmuv3`, `sha2` |
/// | `cortex-a65` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `ssbs`, `v8.1a`, `v8.2a`, `vh` |
/// | `cortex-a65ae` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `ssbs`, `v8.1a`, `v8.2a`, `vh` |
/// | `cortex-a710` | `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `mte`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `ssbs`, `sve`, `sve2`, `sve2-bitperm`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `vh` |
/// | `cortex-a715` | `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `mte`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `spe`, `ssbs`, `sve`, `sve2`, `sve2-bitperm`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `vh` |
/// | `cortex-a72` | `aes`, `crc`, `neon`, `pmuv3`, `sha2` |
/// | `cortex-a720` | `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `mte`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `spe`, `ssbs`, `sve`, `sve2`, `sve2-bitperm`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `v8.7a`, `vh` |
/// | `cortex-a73` | `aes`, `crc`, `neon`, `pmuv3`, `sha2` |
/// | `cortex-a75` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `v8.1a`, `v8.2a`, `vh` |
/// | `cortex-a76` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `ssbs`, `v8.1a`, `v8.2a`, `vh` |
/// | `cortex-a76ae` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `ssbs`, `v8.1a`, `v8.2a`, `vh` |
/// | `cortex-a77` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `ssbs`, `v8.1a`, `v8.2a`, `vh` |
/// | `cortex-a78` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `spe`, `ssbs`, `v8.1a`, `v8.2a`, `vh` |
/// | `cortex-a78c` | `aes`, `crc`, `dotprod`, `dpb`, `flagm`, `fp16`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `spe`, `ssbs`, `v8.1a`, `v8.2a`, `vh` |
/// | `cortex-r82` | `crc`, `dit`, `dotprod`, `dpb`, `fcma`, `fhm`, `flagm`, `fp16`, `jsconv`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `ssbs` |
/// | `cortex-x1` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `spe`, `ssbs`, `v8.1a`, `v8.2a`, `vh` |
/// | `cortex-x1c` | `aes`, `crc`, `dotprod`, `dpb`, `flagm`, `fp16`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sha2`, `spe`, `ssbs`, `v8.1a`, `v8.2a`, `vh` |
/// | `cortex-x2` | `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `mte`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `ssbs`, `sve`, `sve2`, `sve2-bitperm`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `vh` |
/// | `cortex-x3` | `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `mte`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `spe`, `ssbs`, `sve`, `sve2`, `sve2-bitperm`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `vh` |
/// | `cortex-x4` | `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `mte`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `spe`, `ssbs`, `sve`, `sve2`, `sve2-bitperm`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `v8.6a`, `v8.7a`, `vh` |
/// | `cyclone` | `aes`, `neon`, `pmuv3`, `sha2` |
/// | `exynos-m3` | `aes`, `crc`, `neon`, `pmuv3`, `sha2` |
/// | `exynos-m4` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rdm`, `sha2`, `v8.1a`, `v8.2a`, `vh` |
/// | `exynos-m5` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rdm`, `sha2`, `v8.1a`, `v8.2a`, `vh` |
/// | `falkor` | `aes`, `crc`, `neon`, `pmuv3`, `rdm`, `sha2` |
/// | `generic` | `neon` |
/// | `kryo` | `aes`, `crc`, `neon`, `pmuv3`, `sha2` |
/// | `neoverse-512tvb` | `aes`, `bf16`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `i8mm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `rand`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sha2`, `spe`, `ssbs`, `sve`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `vh` |
/// | `neoverse-e1` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `ssbs`, `v8.1a`, `v8.2a`, `vh` |
/// | `neoverse-n1` | `aes`, `crc`, `dotprod`, `dpb`, `fp16`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `spe`, `ssbs`, `v8.1a`, `v8.2a`, `vh` |
/// | `neoverse-n2` | `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `mte`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `ssbs`, `sve`, `sve2`, `sve2-bitperm`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `vh` |
/// | `neoverse-v1` | `aes`, `bf16`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `i8mm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `rand`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sha2`, `spe`, `ssbs`, `sve`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `vh` |
/// | `neoverse-v2` | `bf16`, `bti`, `crc`, `dit`, `dotprod`, `dpb`, `dpb2`, `fcma`, `fhm`, `flagm`, `fp16`, `frintts`, `i8mm`, `jsconv`, `lor`, `lse`, `mte`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `rand`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sb`, `spe`, `ssbs`, `sve`, `sve2`, `sve2-bitperm`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `v8.5a`, `vh` |
/// | `saphira` | `aes`, `crc`, `dit`, `dotprod`, `dpb`, `fcma`, `flagm`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rcpc2`, `rdm`, `sha2`, `spe`, `v8.1a`, `v8.2a`, `v8.3a`, `v8.4a`, `vh` |
/// | `thunderx` | `aes`, `crc`, `neon`, `pmuv3`, `sha2` |
/// | `thunderx2t99` | `aes`, `crc`, `lor`, `lse`, `neon`, `pan`, `rdm`, `sha2`, `v8.1a`, `vh` |
/// | `thunderx3t110` | `aes`, `crc`, `dpb`, `fcma`, `jsconv`, `lor`, `lse`, `neon`, `paca`, `pacg`, `pan`, `pmuv3`, `ras`, `rcpc`, `rdm`, `sha2`, `v8.1a`, `v8.2a`, `v8.3a`, `vh` |
/// | `thunderxt81` | `aes`, `crc`, `neon`, `pmuv3`, `sha2` |
/// | `thunderxt83` | `aes`, `crc`, `neon`, `pmuv3`, `sha2` |
/// | `thunderxt88` | `aes`, `crc`, `neon`, `pmuv3`, `sha2` |
/// | `tsv110` | `aes`, `crc`, `dotprod`, `dpb`, `fcma`, `fhm`, `fp16`, `jsconv`, `lor`, `lse`, `neon`, `pan`, `pmuv3`, `ras`, `rdm`, `sha2`, `spe`, `v8.1a`, `v8.2a`, `vh` |
pub mod aarch64 {}
/// arm documentation
///
/// ## Features
/// | Feature | Description | Also Enables<sup>†</sup> |
/// | ------- | ----------- | ------------------------ |
/// | `aclass` | Is application profile ('A' series). |  |
/// | `aes` | Enable AES support. | `d32`, `neon`, `vfp2`, `vfp3` |
/// | `crc` | Enable support for CRC instructions. |  |
/// | `d32` | Extend FP to 32 double registers. |  |
/// | `dotprod` | Enable support for dot product instructions. | `d32`, `neon`, `vfp2`, `vfp3` |
/// | `dsp` | Supports DSP instructions in ARM and/or Thumb2. |  |
/// | `fp-armv8` | Enable ARMv8 FP. | `d32`, `vfp2`, `vfp3`, `vfp4` |
/// | `i8mm` | Enable Matrix Multiply Int8 Extension. | `d32`, `neon`, `vfp2`, `vfp3` |
/// | `mclass` | Is microcontroller profile ('M' series). |  |
/// | `neon` | Enable NEON instructions. | `d32`, `vfp2`, `vfp3` |
/// | `rclass` | Is realtime profile ('R' series). |  |
/// | `sha2` | Enable SHA1 and SHA256 support. | `d32`, `neon`, `vfp2`, `vfp3` |
/// | `thumb-mode` | Thumb mode. |  |
/// | `thumb2` | Enable Thumb2 instructions. |  |
/// | `trustzone` | Enable support for TrustZone security extensions. |  |
/// | `v5te` | Support ARM v5TE, v5TEj, and v5TExp instructions. |  |
/// | `v6` | Support ARM v6 instructions. | `v5te` |
/// | `v6k` | Support ARM v6k instructions. | `v5te`, `v6` |
/// | `v6t2` | Support ARM v6t2 instructions. | `thumb2`, `v5te`, `v6`, `v6k` |
/// | `v7` | Support ARM v7 instructions. | `thumb2`, `v5te`, `v6`, `v6k`, `v6t2` |
/// | `v8` | Support ARM v8 instructions. | `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7` |
/// | `vfp2` | Enable VFP2 instructions. |  |
/// | `vfp3` | Enable VFP3 instructions. | `d32`, `vfp2` |
/// | `vfp4` | Enable VFP4 instructions. | `d32`, `vfp2`, `vfp3` |
/// | `virtualization` | Supports Virtualization extension. |  |
/// | `crt-static` | Enables C Run-time Libraries to be statically linked. |  |
///
/// <sup>†</sup> This is often empirical, rather than specified in any standard, i.e. all available CPUs with a particular feature also have another feature.
///
/// ## CPUs
/// | CPU | Enabled Features |
/// | --- | -------- |
/// | `arm1020e` | `v5te`, `v6`, `vfp2` |
/// | `arm1020t` | `v5te`, `v6`, `vfp2` |
/// | `arm1022e` | `v5te`, `v6`, `vfp2` |
/// | `arm10e` | `v5te`, `v6`, `vfp2` |
/// | `arm10tdmi` | `v5te`, `v6`, `vfp2` |
/// | `arm1136j-s` | `dsp`, `v5te`, `v6`, `vfp2` |
/// | `arm1136jf-s` | `dsp`, `v5te`, `v6`, `vfp2` |
/// | `arm1156t2-s` | `dsp`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `vfp2` |
/// | `arm1156t2f-s` | `dsp`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `vfp2` |
/// | `arm1176jz-s` | `trustzone`, `v5te`, `v6`, `v6k`, `vfp2` |
/// | `arm1176jzf-s` | `trustzone`, `v5te`, `v6`, `v6k`, `vfp2` |
/// | `arm710t` | `v5te`, `v6`, `vfp2` |
/// | `arm720t` | `v5te`, `v6`, `vfp2` |
/// | `arm7tdmi` | `v5te`, `v6`, `vfp2` |
/// | `arm7tdmi-s` | `v5te`, `v6`, `vfp2` |
/// | `arm8` | `v5te`, `v6`, `vfp2` |
/// | `arm810` | `v5te`, `v6`, `vfp2` |
/// | `arm9` | `v5te`, `v6`, `vfp2` |
/// | `arm920` | `v5te`, `v6`, `vfp2` |
/// | `arm920t` | `v5te`, `v6`, `vfp2` |
/// | `arm922t` | `v5te`, `v6`, `vfp2` |
/// | `arm926ej-s` | `v5te`, `v6`, `vfp2` |
/// | `arm940t` | `v5te`, `v6`, `vfp2` |
/// | `arm946e-s` | `v5te`, `v6`, `vfp2` |
/// | `arm966e-s` | `v5te`, `v6`, `vfp2` |
/// | `arm968e-s` | `v5te`, `v6`, `vfp2` |
/// | `arm9e` | `v5te`, `v6`, `vfp2` |
/// | `arm9tdmi` | `v5te`, `v6`, `vfp2` |
/// | `cortex-a12` | `aclass`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2`, `virtualization` |
/// | `cortex-a15` | `aclass`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2`, `virtualization` |
/// | `cortex-a17` | `aclass`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2`, `virtualization` |
/// | `cortex-a32` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a35` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a5` | `aclass`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-a53` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a55` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a57` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a7` | `aclass`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2`, `virtualization` |
/// | `cortex-a710` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a72` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a73` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a75` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a76` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a76ae` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a77` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a78` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a78c` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-a8` | `aclass`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-a9` | `aclass`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-m0` | `mclass`, `thumb-mode`, `v5te`, `v6`, `vfp2` |
/// | `cortex-m0plus` | `mclass`, `thumb-mode`, `v5te`, `v6`, `vfp2` |
/// | `cortex-m1` | `mclass`, `thumb-mode`, `v5te`, `v6`, `vfp2` |
/// | `cortex-m23` | `mclass`, `thumb-mode`, `v5te`, `v6`, `vfp2` |
/// | `cortex-m3` | `mclass`, `thumb-mode`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-m33` | `dsp`, `mclass`, `thumb-mode`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-m35p` | `dsp`, `mclass`, `thumb-mode`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-m4` | `dsp`, `mclass`, `thumb-mode`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-m52` | `dsp`, `mclass`, `thumb-mode`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-m55` | `dsp`, `mclass`, `thumb-mode`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-m7` | `dsp`, `mclass`, `thumb-mode`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-m85` | `dsp`, `mclass`, `thumb-mode`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-r4` | `dsp`, `rclass`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-r4f` | `dsp`, `rclass`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-r5` | `dsp`, `rclass`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-r52` | `crc`, `dsp`, `rclass`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-r7` | `dsp`, `rclass`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-r8` | `dsp`, `rclass`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `cortex-x1` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cortex-x1c` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `cyclone` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `ep9312` | `v5te`, `v6`, `vfp2` |
/// | `exynos-m3` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `exynos-m4` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `exynos-m5` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `generic` | `v5te`, `v6`, `vfp2` |
/// | `iwmmxt` | `v5te`, `v6`, `vfp2` |
/// | `krait` | `aclass`, `dsp`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `kryo` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `mpcore` | `v5te`, `v6`, `v6k`, `vfp2` |
/// | `mpcorenovfp` | `v5te`, `v6`, `v6k`, `vfp2` |
/// | `neoverse-n1` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `neoverse-n2` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `neoverse-v1` | `aclass`, `crc`, `dsp`, `thumb2`, `trustzone`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `v8`, `vfp2`, `virtualization` |
/// | `sc000` | `mclass`, `thumb-mode`, `v5te`, `v6`, `vfp2` |
/// | `sc300` | `mclass`, `thumb-mode`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `strongarm` | `v5te`, `v6`, `vfp2` |
/// | `strongarm110` | `v5te`, `v6`, `vfp2` |
/// | `strongarm1100` | `v5te`, `v6`, `vfp2` |
/// | `strongarm1110` | `v5te`, `v6`, `vfp2` |
/// | `swift` | `aclass`, `dsp`, `thumb2`, `v5te`, `v6`, `v6k`, `v6t2`, `v7`, `vfp2` |
/// | `xscale` | `v5te`, `v6`, `vfp2` |
pub mod arm {}
/// bpf documentation
///
/// ## Features
/// | Feature | Description | Also Enables<sup>†</sup> |
/// | ------- | ----------- | ------------------------ |
/// | `alu32` | Enable ALU32 instructions. |  |
/// | `crt-static` | Enables C Run-time Libraries to be statically linked. |  |
///
/// <sup>†</sup> This is often empirical, rather than specified in any standard, i.e. all available CPUs with a particular feature also have another feature.
///
/// ## CPUs
/// | CPU | Enabled Features |
/// | --- | -------- |
/// | `generic` |  |
/// | `probe` |  |
/// | `v1` |  |
/// | `v2` |  |
/// | `v3` | `alu32` |
/// | `v4` | `alu32` |
pub mod bpf {}
/// hexagon documentation
///
/// ## Features
/// | Feature | Description | Also Enables<sup>†</sup> |
/// | ------- | ----------- | ------------------------ |
/// | `hvx` | Hexagon HVX instructions. |  |
/// | `hvx-length128b` | Hexagon HVX 128B instructions. | `hvx` |
/// | `crt-static` | Enables C Run-time Libraries to be statically linked. |  |
///
/// <sup>†</sup> This is often empirical, rather than specified in any standard, i.e. all available CPUs with a particular feature also have another feature.
///
/// ## CPUs
/// | CPU | Enabled Features |
/// | --- | -------- |
/// | `generic` | `hvx`, `hvx-length128b` |
/// | `hexagonv5` | `hvx`, `hvx-length128b` |
/// | `hexagonv55` | `hvx`, `hvx-length128b` |
/// | `hexagonv60` | `hvx`, `hvx-length128b` |
/// | `hexagonv62` | `hvx`, `hvx-length128b` |
/// | `hexagonv65` | `hvx`, `hvx-length128b` |
/// | `hexagonv66` | `hvx`, `hvx-length128b` |
/// | `hexagonv67` | `hvx`, `hvx-length128b` |
/// | `hexagonv67t` | `hvx`, `hvx-length128b` |
/// | `hexagonv68` | `hvx`, `hvx-length128b` |
/// | `hexagonv69` | `hvx`, `hvx-length128b` |
/// | `hexagonv71` | `hvx`, `hvx-length128b` |
/// | `hexagonv71t` | `hvx`, `hvx-length128b` |
/// | `hexagonv73` | `hvx`, `hvx-length128b` |
pub mod hexagon {}
/// mips documentation
///
/// ## Features
/// | Feature | Description | Also Enables<sup>†</sup> |
/// | ------- | ----------- | ------------------------ |
/// | `fp64` | Support 64-bit FP registers. |  |
/// | `msa` | Mips MSA ASE. | `fp64` |
/// | `virt` | Mips Virtualization ASE. | `fp64` |
/// | `crt-static` | Enables C Run-time Libraries to be statically linked. | `fp64` |
///
/// <sup>†</sup> This is often empirical, rather than specified in any standard, i.e. all available CPUs with a particular feature also have another feature.
///
/// ## CPUs
/// | CPU | Enabled Features |
/// | --- | -------- |
/// | `generic` | `fp64` |
/// | `mips1` | `fp64` |
/// | `mips2` | `fp64` |
/// | `mips3` | `fp64` |
/// | `mips32` | `fp64` |
/// | `mips32r2` | `fp64` |
/// | `mips32r3` | `fp64` |
/// | `mips32r5` | `fp64` |
/// | `mips32r6` | `fp64` |
/// | `mips4` | `fp64` |
/// | `mips5` | `fp64` |
/// | `mips64` | `fp64` |
/// | `mips64r2` | `fp64` |
/// | `mips64r3` | `fp64` |
/// | `mips64r5` | `fp64` |
/// | `mips64r6` | `fp64` |
/// | `octeon` | `fp64` |
/// | `octeon+` | `fp64` |
/// | `p5600` | `fp64` |
pub mod mips {}
/// powerpc documentation
///
/// ## Features
/// | Feature | Description | Also Enables<sup>†</sup> |
/// | ------- | ----------- | ------------------------ |
/// | `altivec` | Enable Altivec instructions. |  |
/// | `power10-vector` | Enable POWER10 vector instructions. | `altivec`, `power8-altivec`, `power8-vector`, `power9-altivec`, `power9-vector`, `vsx` |
/// | `power8-altivec` | Enable POWER8 Altivec instructions. | `altivec` |
/// | `power8-vector` | Enable POWER8 vector instructions. | `altivec`, `power8-altivec`, `vsx` |
/// | `power9-altivec` | Enable POWER9 Altivec instructions. | `altivec`, `power8-altivec` |
/// | `power9-vector` | Enable POWER9 vector instructions. | `altivec`, `power8-altivec`, `power8-vector`, `power9-altivec`, `vsx` |
/// | `vsx` | Enable VSX instructions. | `altivec` |
/// | `crt-static` | Enables C Run-time Libraries to be statically linked. |  |
///
/// <sup>†</sup> This is often empirical, rather than specified in any standard, i.e. all available CPUs with a particular feature also have another feature.
///
/// ## CPUs
/// | CPU | Enabled Features |
/// | --- | -------- |
/// | `440` |  |
/// | `450` |  |
/// | `601` |  |
/// | `602` |  |
/// | `603` |  |
/// | `603e` |  |
/// | `603ev` |  |
/// | `604` |  |
/// | `604e` |  |
/// | `620` |  |
/// | `7400` | `altivec` |
/// | `7450` | `altivec` |
/// | `750` |  |
/// | `970` | `altivec` |
/// | `a2` |  |
/// | `e500` |  |
/// | `e500mc` |  |
/// | `e5500` |  |
/// | `future` | `altivec`, `power10-vector`, `power8-altivec`, `power8-vector`, `power9-altivec`, `power9-vector`, `vsx` |
/// | `g3` |  |
/// | `g4` | `altivec` |
/// | `g4+` | `altivec` |
/// | `g5` | `altivec` |
/// | `generic` |  |
/// | `ppc` |  |
/// | `ppc32` |  |
/// | `ppc64` | `altivec` |
/// | `ppc64le` | `altivec`, `power8-altivec`, `power8-vector`, `vsx` |
/// | `pwr10` | `altivec`, `power10-vector`, `power8-altivec`, `power8-vector`, `power9-altivec`, `power9-vector`, `vsx` |
/// | `pwr3` | `altivec` |
/// | `pwr4` | `altivec` |
/// | `pwr5` | `altivec` |
/// | `pwr5x` | `altivec` |
/// | `pwr6` | `altivec` |
/// | `pwr6x` | `altivec` |
/// | `pwr7` | `altivec`, `vsx` |
/// | `pwr8` | `altivec`, `power8-altivec`, `power8-vector`, `vsx` |
/// | `pwr9` | `altivec`, `power8-altivec`, `power8-vector`, `power9-altivec`, `power9-vector`, `vsx` |
pub mod powerpc {}
/// riscv documentation
///
/// ## Features
/// | Feature | Description | Also Enables<sup>†</sup> |
/// | ------- | ----------- | ------------------------ |
/// | `a` | 'A' (Atomic Instructions). |  |
/// | `c` | 'C' (Compressed Instructions). |  |
/// | `d` | 'D' (Double-Precision Floating-Point). | `f` |
/// | `e` | Implements RV{32,64}E (provides 16 rather than 32 GPRs). |  |
/// | `f` | 'F' (Single-Precision Floating-Point). |  |
/// | `fast-unaligned-access` | Has reasonably performant unaligned loads and stores (both scalar and vector). |  |
/// | `m` | 'M' (Integer Multiplication and Division). |  |
/// | `relax` | Enable Linker relaxation.. |  |
/// | `v` | 'V' (Vector Extension for Application Processors). | `d`, `f` |
/// | `zba` | 'Zba' (Address Generation Instructions). |  |
/// | `zbb` | 'Zbb' (Basic Bit-Manipulation). |  |
/// | `zbc` | 'Zbc' (Carry-Less Multiplication). |  |
/// | `zbkb` | 'Zbkb' (Bitmanip instructions for Cryptography). |  |
/// | `zbkc` | 'Zbkc' (Carry-less multiply instructions for Cryptography). |  |
/// | `zbkx` | 'Zbkx' (Crossbar permutation instructions). |  |
/// | `zbs` | 'Zbs' (Single-Bit Instructions). |  |
/// | `zdinx` | 'Zdinx' (Double in Integer). | `zfinx` |
/// | `zfh` | 'Zfh' (Half-Precision Floating-Point). | `f`, `zfhmin` |
/// | `zfhmin` | 'Zfhmin' (Half-Precision Floating-Point Minimal). | `f` |
/// | `zfinx` | 'Zfinx' (Float in Integer). |  |
/// | `zhinx` | 'Zhinx' (Half Float in Integer). | `zfinx`, `zhinxmin` |
/// | `zhinxmin` | 'Zhinxmin' (Half Float in Integer Minimal). | `zfinx` |
/// | `zk` | 'Zk' (Standard scalar cryptography extension). | `zbkb`, `zbkc`, `zbkx`, `zkn`, `zknd`, `zkne`, `zknh`, `zkr`, `zkt` |
/// | `zkn` | 'Zkn' (NIST Algorithm Suite). | `zbkb`, `zbkc`, `zbkx`, `zknd`, `zkne`, `zknh` |
/// | `zknd` | 'Zknd' (NIST Suite: AES Decryption). |  |
/// | `zkne` | 'Zkne' (NIST Suite: AES Encryption). |  |
/// | `zknh` | 'Zknh' (NIST Suite: Hash Function Instructions). |  |
/// | `zkr` | 'Zkr' (Entropy Source Extension). |  |
/// | `zks` | 'Zks' (ShangMi Algorithm Suite). | `zbkb`, `zbkc`, `zbkx`, `zksed`, `zksh` |
/// | `zksed` | 'Zksed' (ShangMi Suite: SM4 Block Cipher Instructions). |  |
/// | `zksh` | 'Zksh' (ShangMi Suite: SM3 Hash Function Instructions). |  |
/// | `zkt` | 'Zkt' (Data Independent Execution Latency). |  |
/// | `crt-static` | Enables C Run-time Libraries to be statically linked. |  |
///
/// <sup>†</sup> This is often empirical, rather than specified in any standard, i.e. all available CPUs with a particular feature also have another feature.
///
/// ## CPUs
/// | CPU | Enabled Features |
/// | --- | -------- |
/// | `generic` | `a`, `c`, `d`, `f`, `m` |
/// | `generic-rv32` | `a`, `c`, `d`, `f`, `m` |
/// | `generic-rv64` | `a`, `c`, `d`, `f`, `m` |
/// | `rocket` | `a`, `c`, `d`, `f`, `m` |
/// | `rocket-rv32` | `a`, `c`, `d`, `f`, `m` |
/// | `rocket-rv64` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-7-series` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-e20` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-e21` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-e24` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-e31` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-e34` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-e76` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-p450` | `a`, `c`, `d`, `f`, `fast-unaligned-access`, `m`, `zba`, `zbb`, `zbs`, `zfhmin` |
/// | `sifive-p670` | `a`, `c`, `d`, `f`, `fast-unaligned-access`, `m`, `v`, `zba`, `zbb`, `zbs`, `zfhmin` |
/// | `sifive-s21` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-s51` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-s54` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-s76` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-u54` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-u74` | `a`, `c`, `d`, `f`, `m` |
/// | `sifive-x280` | `a`, `c`, `d`, `f`, `m`, `v`, `zba`, `zbb`, `zfh`, `zfhmin` |
/// | `syntacore-scr1-base` | `a`, `c`, `d`, `f`, `m` |
/// | `syntacore-scr1-max` | `a`, `c`, `d`, `f`, `m` |
/// | `veyron-v1` | `a`, `c`, `d`, `f`, `m`, `zba`, `zbb`, `zbc`, `zbs` |
/// | `xiangshan-nanhu` | `a`, `c`, `d`, `f`, `m`, `zba`, `zbb`, `zbc`, `zbkb`, `zbkc`, `zbkx`, `zbs`, `zkn`, `zknd`, `zkne`, `zknh`, `zksed`, `zksh` |
pub mod riscv {}
/// wasm documentation
///
/// ## Features
/// | Feature | Description | Also Enables<sup>†</sup> |
/// | ------- | ----------- | ------------------------ |
/// | `atomics` | Enable Atomics. |  |
/// | `bulk-memory` | Enable bulk memory operations. |  |
/// | `exception-handling` | Enable Wasm exception handling. |  |
/// | `multivalue` | Enable multivalue blocks, instructions, and functions. |  |
/// | `mutable-globals` | Enable mutable globals. |  |
/// | `nontrapping-fptoint` | Enable non-trapping float-to-int conversion operators. |  |
/// | `reference-types` | Enable reference types. |  |
/// | `relaxed-simd` | Enable relaxed-simd instructions. |  |
/// | `sign-ext` | Enable sign extension operators. |  |
/// | `simd128` | Enable 128-bit SIMD. |  |
/// | `crt-static` | Enables C Run-time Libraries to be statically linked. |  |
///
/// <sup>†</sup> This is often empirical, rather than specified in any standard, i.e. all available CPUs with a particular feature also have another feature.
///
/// ## CPUs
/// | CPU | Enabled Features |
/// | --- | -------- |
/// | `bleeding-edge` | `atomics`, `bulk-memory`, `mutable-globals`, `nontrapping-fptoint`, `sign-ext`, `simd128` |
/// | `generic` | `mutable-globals`, `sign-ext` |
/// | `mvp` |  |
pub mod wasm {}
/// x86 documentation
///
/// ## Features
/// | Feature | Description | Also Enables<sup>†</sup> |
/// | ------- | ----------- | ------------------------ |
/// | `adx` | Support ADX instructions. |  |
/// | `aes` | Enable AES instructions. | `sse`, `sse2` |
/// | `avx` | Enable AVX instructions. | `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx2` | Enable AVX2 instructions. | `avx`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512bf16` | Support bfloat16 floating point. | `avx`, `avx2`, `avx512bw`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512bitalg` | Enable AVX-512 Bit Algorithms. | `avx`, `avx2`, `avx512bw`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512bw` | Enable AVX-512 Byte and Word Instructions. | `avx`, `avx2`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512cd` | Enable AVX-512 Conflict Detection Instructions. | `avx`, `avx2`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512dq` | Enable AVX-512 Doubleword and Quadword Instructions. | `avx`, `avx2`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512er` | Enable AVX-512 Exponential and Reciprocal Instructions. | `avx`, `avx2`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512f` | Enable AVX-512 instructions. | `avx`, `avx2`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512fp16` | Support 16-bit floating point. | `avx`, `avx2`, `avx512bw`, `avx512dq`, `avx512f`, `avx512vl`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512ifma` | Enable AVX-512 Integer Fused Multiple-Add. | `avx`, `avx2`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512pf` | Enable AVX-512 PreFetch Instructions. | `avx`, `avx2`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512vbmi` | Enable AVX-512 Vector Byte Manipulation Instructions. | `avx`, `avx2`, `avx512bw`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512vbmi2` | Enable AVX-512 further Vector Byte Manipulation Instructions. | `avx`, `avx2`, `avx512bw`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512vl` | Enable AVX-512 Vector Length eXtensions. | `avx`, `avx2`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512vnni` | Enable AVX-512 Vector Neural Network Instructions. | `avx`, `avx2`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512vp2intersect` | Enable AVX-512 vp2intersect. | `avx`, `avx2`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `avx512vpopcntdq` | Enable AVX-512 Population Count Instructions. | `avx`, `avx2`, `avx512f`, `f16c`, `fma`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `bmi1` | Support BMI instructions. |  |
/// | `bmi2` | Support BMI2 instructions. |  |
/// | `cmpxchg16b` | 64-bit with cmpxchg16b (this is true for most x86-64 chips, but not the first AMD chips). |  |
/// | `ermsb` | REP MOVS/STOS are fast. |  |
/// | `f16c` | Support 16-bit floating point conversion instructions. | `avx`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `fma` | Enable three-operand fused multiple-add. | `avx`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `fxsr` | Support fxsave/fxrestore instructions. |  |
/// | `gfni` | Enable Galois Field Arithmetic Instructions. | `sse`, `sse2` |
/// | `lahfsahf` | Support LAHF and SAHF instructions in 64-bit mode. |  |
/// | `lzcnt` | Support LZCNT instruction. |  |
/// | `movbe` | Support MOVBE instruction. |  |
/// | `pclmulqdq` | Enable packed carry-less multiplication instructions. | `sse`, `sse2` |
/// | `popcnt` | Support POPCNT instruction. |  |
/// | `prfchw` | Support PRFCHW instructions. |  |
/// | `rdrand` | Support RDRAND instruction. |  |
/// | `rdseed` | Support RDSEED instruction. |  |
/// | `rtm` | Support RTM instructions. |  |
/// | `sha` | Enable SHA instructions. | `sse`, `sse2` |
/// | `sse` | Enable SSE instructions. |  |
/// | `sse2` | Enable SSE2 instructions. | `sse` |
/// | `sse3` | Enable SSE3 instructions. | `sse`, `sse2` |
/// | `sse4.1` | Enable SSE 4.1 instructions. | `sse`, `sse2`, `sse3`, `ssse3` |
/// | `sse4.2` | Enable SSE 4.2 instructions. | `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `sse4a` | Support SSE 4a instructions. | `sse`, `sse2`, `sse3` |
/// | `ssse3` | Enable SSSE3 instructions. | `sse`, `sse2`, `sse3` |
/// | `tbm` | Enable TBM instructions. |  |
/// | `vaes` | Promote selected AES instructions to AVX512/AVX registers. | `aes`, `avx`, `avx2`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `vpclmulqdq` | Enable vpclmulqdq instructions. | `avx`, `pclmulqdq`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `xsave` | Support xsave instructions. |  |
/// | `xsavec` | Support xsavec instructions. | `xsave` |
/// | `xsaveopt` | Support xsaveopt instructions. | `xsave` |
/// | `xsaves` | Support xsaves instructions. | `xsave` |
/// | `crt-static` | Enables C Run-time Libraries to be statically linked. |  |
///
/// <sup>†</sup> This is often empirical, rather than specified in any standard, i.e. all available CPUs with a particular feature also have another feature.
///
/// ## CPUs
/// | CPU | Enabled Features |
/// | --- | -------- |
/// | `alderlake` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `amdfam10` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `lzcnt`, `popcnt`, `prfchw`, `sse`, `sse2`, `sse3`, `sse4a` |
/// | `arrowlake` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `arrowlake-s` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `arrowlake_s` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `athlon` | `sse`, `sse2` |
/// | `athlon-4` | `fxsr`, `sse`, `sse2` |
/// | `athlon-fx` | `fxsr`, `sse`, `sse2` |
/// | `athlon-mp` | `fxsr`, `sse`, `sse2` |
/// | `athlon-tbird` | `sse`, `sse2` |
/// | `athlon-xp` | `fxsr`, `sse`, `sse2` |
/// | `athlon64` | `fxsr`, `sse`, `sse2` |
/// | `athlon64-sse3` | `cmpxchg16b`, `fxsr`, `sse`, `sse2`, `sse3` |
/// | `atom` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `movbe`, `sse`, `sse2`, `sse3`, `ssse3` |
/// | `atom_sse4_2` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3` |
/// | `atom_sse4_2_movbe` | `aes`, `cmpxchg16b`, `fxsr`, `lahfsahf`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `barcelona` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `lzcnt`, `popcnt`, `prfchw`, `sse`, `sse2`, `sse3`, `sse4a` |
/// | `bdver1` | `aes`, `avx`, `cmpxchg16b`, `fxsr`, `lahfsahf`, `lzcnt`, `pclmulqdq`, `popcnt`, `prfchw`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `sse4a`, `ssse3`, `xsave` |
/// | `bdver2` | `aes`, `avx`, `bmi1`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `pclmulqdq`, `popcnt`, `prfchw`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `sse4a`, `ssse3`, `tbm`, `xsave` |
/// | `bdver3` | `aes`, `avx`, `bmi1`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `pclmulqdq`, `popcnt`, `prfchw`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `sse4a`, `ssse3`, `tbm`, `xsave`, `xsaveopt` |
/// | `bdver4` | `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `sse4a`, `ssse3`, `tbm`, `xsave`, `xsaveopt` |
/// | `bonnell` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `movbe`, `sse`, `sse2`, `sse3`, `ssse3` |
/// | `broadwell` | `adx`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `btver1` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `lzcnt`, `popcnt`, `prfchw`, `sse`, `sse2`, `sse3`, `sse4a`, `ssse3` |
/// | `btver2` | `aes`, `avx`, `bmi1`, `cmpxchg16b`, `f16c`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `sse4a`, `ssse3`, `xsave`, `xsaveopt` |
/// | `c3` | `sse`, `sse2` |
/// | `c3-2` | `fxsr`, `sse`, `sse2` |
/// | `cannonlake` | `adx`, `aes`, `avx`, `avx2`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512ifma`, `avx512vbmi`, `avx512vl`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `cascadelake` | `adx`, `aes`, `avx`, `avx2`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512vl`, `avx512vnni`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `clearwaterforest` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `cooperlake` | `adx`, `aes`, `avx`, `avx2`, `avx512bf16`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512vl`, `avx512vnni`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `core-avx-i` | `avx`, `cmpxchg16b`, `f16c`, `fxsr`, `lahfsahf`, `pclmulqdq`, `popcnt`, `rdrand`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `core-avx2` | `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `rdrand`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `core2` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `sse`, `sse2`, `sse3`, `ssse3` |
/// | `core_2_duo_sse4_1` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `core_2_duo_ssse3` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `sse`, `sse2`, `sse3`, `ssse3` |
/// | `core_2nd_gen_avx` | `avx`, `cmpxchg16b`, `fxsr`, `lahfsahf`, `pclmulqdq`, `popcnt`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `core_3rd_gen_avx` | `avx`, `cmpxchg16b`, `f16c`, `fxsr`, `lahfsahf`, `pclmulqdq`, `popcnt`, `rdrand`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `core_4th_gen_avx` | `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `rdrand`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `core_4th_gen_avx_tsx` | `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `rdrand`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `core_5th_gen_avx` | `adx`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `core_5th_gen_avx_tsx` | `adx`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `core_aes_pclmulqdq` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `pclmulqdq`, `popcnt`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3` |
/// | `core_i7_sse4_2` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `popcnt`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3` |
/// | `corei7` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `popcnt`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3` |
/// | `corei7-avx` | `avx`, `cmpxchg16b`, `fxsr`, `lahfsahf`, `pclmulqdq`, `popcnt`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `emeraldrapids` | `adx`, `aes`, `avx`, `avx2`, `avx512bf16`, `avx512bitalg`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512fp16`, `avx512ifma`, `avx512vbmi`, `avx512vbmi2`, `avx512vl`, `avx512vnni`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `generic` | `sse`, `sse2` |
/// | `geode` | `sse`, `sse2` |
/// | `goldmont` | `aes`, `cmpxchg16b`, `fxsr`, `lahfsahf`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `goldmont-plus` | `aes`, `cmpxchg16b`, `fxsr`, `lahfsahf`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `goldmont_plus` | `aes`, `cmpxchg16b`, `fxsr`, `lahfsahf`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `gracemont` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `grandridge` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `graniterapids` | `adx`, `aes`, `avx`, `avx2`, `avx512bf16`, `avx512bitalg`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512fp16`, `avx512ifma`, `avx512vbmi`, `avx512vbmi2`, `avx512vl`, `avx512vnni`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `graniterapids-d` | `adx`, `aes`, `avx`, `avx2`, `avx512bf16`, `avx512bitalg`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512fp16`, `avx512ifma`, `avx512vbmi`, `avx512vbmi2`, `avx512vl`, `avx512vnni`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `graniterapids_d` | `adx`, `aes`, `avx`, `avx2`, `avx512bf16`, `avx512bitalg`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512fp16`, `avx512ifma`, `avx512vbmi`, `avx512vbmi2`, `avx512vl`, `avx512vnni`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `haswell` | `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `rdrand`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `i386` | `sse`, `sse2` |
/// | `i486` | `sse`, `sse2` |
/// | `i586` | `sse`, `sse2` |
/// | `i686` | `sse`, `sse2` |
/// | `icelake-client` | `adx`, `aes`, `avx`, `avx2`, `avx512bitalg`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512ifma`, `avx512vbmi`, `avx512vbmi2`, `avx512vl`, `avx512vnni`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `icelake-server` | `adx`, `aes`, `avx`, `avx2`, `avx512bitalg`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512ifma`, `avx512vbmi`, `avx512vbmi2`, `avx512vl`, `avx512vnni`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `icelake_client` | `adx`, `aes`, `avx`, `avx2`, `avx512bitalg`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512ifma`, `avx512vbmi`, `avx512vbmi2`, `avx512vl`, `avx512vnni`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `icelake_server` | `adx`, `aes`, `avx`, `avx2`, `avx512bitalg`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512ifma`, `avx512vbmi`, `avx512vbmi2`, `avx512vl`, `avx512vnni`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `ivybridge` | `avx`, `cmpxchg16b`, `f16c`, `fxsr`, `lahfsahf`, `pclmulqdq`, `popcnt`, `rdrand`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `k6` | `sse`, `sse2` |
/// | `k6-2` | `sse`, `sse2` |
/// | `k6-3` | `sse`, `sse2` |
/// | `k8` | `fxsr`, `sse`, `sse2` |
/// | `k8-sse3` | `cmpxchg16b`, `fxsr`, `sse`, `sse2`, `sse3` |
/// | `knl` | `adx`, `aes`, `avx`, `avx2`, `avx512cd`, `avx512er`, `avx512f`, `avx512pf`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `knm` | `adx`, `aes`, `avx`, `avx2`, `avx512cd`, `avx512er`, `avx512f`, `avx512pf`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `lakemont` | `sse`, `sse2` |
/// | `lunarlake` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `meteorlake` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `mic_avx512` | `adx`, `aes`, `avx`, `avx2`, `avx512cd`, `avx512er`, `avx512f`, `avx512pf`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `nehalem` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `popcnt`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3` |
/// | `nocona` | `cmpxchg16b`, `fxsr`, `sse`, `sse2`, `sse3` |
/// | `opteron` | `fxsr`, `sse`, `sse2` |
/// | `opteron-sse3` | `cmpxchg16b`, `fxsr`, `sse`, `sse2`, `sse3` |
/// | `pantherlake` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `penryn` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `sse`, `sse2`, `sse3`, `sse4.1`, `ssse3` |
/// | `pentium` | `sse`, `sse2` |
/// | `pentium-m` | `fxsr`, `sse`, `sse2` |
/// | `pentium-mmx` | `sse`, `sse2` |
/// | `pentium2` | `fxsr`, `sse`, `sse2` |
/// | `pentium3` | `fxsr`, `sse`, `sse2` |
/// | `pentium3m` | `fxsr`, `sse`, `sse2` |
/// | `pentium4` | `fxsr`, `sse`, `sse2` |
/// | `pentium4m` | `fxsr`, `sse`, `sse2` |
/// | `pentium_4` | `fxsr`, `sse`, `sse2` |
/// | `pentium_4_sse3` | `fxsr`, `sse`, `sse2`, `sse3` |
/// | `pentium_ii` | `fxsr`, `sse`, `sse2` |
/// | `pentium_iii` | `fxsr`, `sse`, `sse2` |
/// | `pentium_iii_no_xmm_regs` | `fxsr`, `sse`, `sse2` |
/// | `pentium_m` | `fxsr`, `sse`, `sse2` |
/// | `pentium_mmx` | `sse`, `sse2` |
/// | `pentium_pro` | `sse`, `sse2` |
/// | `pentiumpro` | `sse`, `sse2` |
/// | `prescott` | `fxsr`, `sse`, `sse2`, `sse3` |
/// | `raptorlake` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `rocketlake` | `adx`, `aes`, `avx`, `avx2`, `avx512bitalg`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512ifma`, `avx512vbmi`, `avx512vbmi2`, `avx512vl`, `avx512vnni`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `sandybridge` | `avx`, `cmpxchg16b`, `fxsr`, `lahfsahf`, `pclmulqdq`, `popcnt`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsaveopt` |
/// | `sapphirerapids` | `adx`, `aes`, `avx`, `avx2`, `avx512bf16`, `avx512bitalg`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512fp16`, `avx512ifma`, `avx512vbmi`, `avx512vbmi2`, `avx512vl`, `avx512vnni`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `sierraforest` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `silvermont` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3` |
/// | `skx` | `adx`, `aes`, `avx`, `avx2`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512vl`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `skylake` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `skylake-avx512` | `adx`, `aes`, `avx`, `avx2`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512vl`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `skylake_avx512` | `adx`, `aes`, `avx`, `avx2`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512vl`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `slm` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3` |
/// | `tigerlake` | `adx`, `aes`, `avx`, `avx2`, `avx512bitalg`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512ifma`, `avx512vbmi`, `avx512vbmi2`, `avx512vl`, `avx512vnni`, `avx512vp2intersect`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `ermsb`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `tremont` | `aes`, `cmpxchg16b`, `fxsr`, `gfni`, `lahfsahf`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `westmere` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `pclmulqdq`, `popcnt`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3` |
/// | `winchip-c6` | `sse`, `sse2` |
/// | `winchip2` | `sse`, `sse2` |
/// | `x86-64` | `fxsr`, `sse`, `sse2` |
/// | `x86-64-v2` | `cmpxchg16b`, `fxsr`, `lahfsahf`, `popcnt`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3` |
/// | `x86-64-v3` | `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `popcnt`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave` |
/// | `x86-64-v4` | `avx`, `avx2`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512vl`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `popcnt`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `ssse3`, `xsave` |
/// | `yonah` | `fxsr`, `sse`, `sse2`, `sse3` |
/// | `znver1` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `sse4a`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `znver2` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `sse4a`, `ssse3`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `znver3` | `adx`, `aes`, `avx`, `avx2`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `sse4a`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
/// | `znver4` | `adx`, `aes`, `avx`, `avx2`, `avx512bf16`, `avx512bitalg`, `avx512bw`, `avx512cd`, `avx512dq`, `avx512f`, `avx512ifma`, `avx512vbmi`, `avx512vbmi2`, `avx512vl`, `avx512vnni`, `avx512vpopcntdq`, `bmi1`, `bmi2`, `cmpxchg16b`, `f16c`, `fma`, `fxsr`, `gfni`, `lahfsahf`, `lzcnt`, `movbe`, `pclmulqdq`, `popcnt`, `prfchw`, `rdrand`, `rdseed`, `sha`, `sse`, `sse2`, `sse3`, `sse4.1`, `sse4.2`, `sse4a`, `ssse3`, `vaes`, `vpclmulqdq`, `xsave`, `xsavec`, `xsaveopt`, `xsaves` |
pub mod x86 {}
