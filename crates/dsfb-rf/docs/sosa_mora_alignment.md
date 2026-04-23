# SOSA / MORA Alignment Assessment

**DSFB-RF Structural Semiotics Engine**

## References

- **SOSA™**: Sensor Open Systems Architecture, The Open Group
- **MORA**: Modular Open RF Architecture, US DoD
- **CMOSS**: C5ISR/EW Modular Open Suite of Standards

## SOSA Alignment Summary

| SOSA Requirement               | DSFB-RF Compliance                                  |
|--------------------------------|-----------------------------------------------------|
| Software-defined processing    | ✓ Pure Rust software, no hardware coupling           |
| Vendor-neutral interfaces      | ✓ `&[f32]` immutable slice input, JSON output        |
| Modular, replaceable components| ✓ Each pipeline stage is an independent module        |
| Standard data transport        | ✓ VITA 49.2 VRT context consumption supported        |
| No vendor lock-in              | ✓ Apache-2.0 reference implementation                |
| Hardware abstraction           | ✓ `no_std`/`no_alloc` — runs on any target with Rust |

## MORA Software Resource Characterization

DSFB-RF is positioned as a MORA-compliant **Software Resource** within the
RF signal processing chain:

### Resource Type: Observer/Interpreter (Read-Only)

```
┌─────────────────────────────────────────────────────────┐
│  MORA RF Processing Chain (existing, unchanged)         │
│                                                         │
│  [Antenna] → [LNA] → [ADC] → [DDC] → [Demod/CFAR]     │
│                                           │             │
│                                    ┌──────┴──────┐      │
│                                    │  IQ Residual │      │
│                                    │  Side-Channel│      │
│                                    └──────┬──────┘      │
└───────────────────────────────────────────┼─────────────┘
                                            │ (read-only tap)
                        ┌───────────────────▼───────────────┐
                        │  DSFB-RF Software Resource         │
                        │  (MORA-aligned Observer Layer)     │
                        │                                    │
                        │  Input:  &[f32] residual norms     │
                        │  Output: PolicyDecision + Episode  │
                        │  Write path: NONE                  │
                        │  Upstream modification: NONE       │
                        └────────────────────────────────────┘
```

### Interface Contract (SOSA/MORA-Compatible)

1. **Input interface**: `ResidualSource` trait (`&[f32]` immutable borrow)
   - Compatible with VITA 49.2 VRT data packet payload
   - Zero-copy: reads directly from DMA buffers
   - No control-plane interface required

2. **Output interface**: `PolicyDecision` enum + `ObservationResult` struct
   - Mappable to SigMF annotations for visualization
   - Exportable as JSON for C2 integration
   - No feedback path into upstream processing

3. **Context interface**: `PlatformContext` struct
   - Populated from VITA 49.2 context extension packets
   - Includes SNR estimate, waveform state, guard timing
   - Read-only: DSFB never writes context packets

### CMOSS Profile

| CMOSS Layer          | DSFB-RF Role                              |
|---------------------|-------------------------------------------|
| Physical (Slot)     | Not applicable — software only             |
| Transport (VPX/PCIe)| Not applicable — software only             |
| Data (VITA 49.2)    | Consumer (reads VRT context packets)       |
| Control (VITA 49.0) | None — no control plane interaction        |
| Application         | Structural observer + advisory output      |

## Deployment Scenarios

### Scenario 1: SOSA-Aligned EW Suite

DSFB-RF deploys as an additional processing element on the SOSA backplane,
tapping the IQ residual stream from existing signal processing cards.
No modification to existing processing elements. Removable without impact.

### Scenario 2: MORA Ground Station

DSFB-RF runs on the general-purpose processor (GPP) blade alongside
existing spectrum monitoring software. Reads residual streams from the
DDC/channelizer output via shared memory. Advisory output routed to
operator display via standard message bus.

### Scenario 3: Embedded Tactical Radio

DSFB-RF runs on the ARM Cortex-M4F or RISC-V softcore within the
radio's FPGA fabric. `no_std`/`no_alloc` deployment, 504 bytes stack.
Reads PLL phase error and AGC gain error from register-mapped I/O.
Advisory output via UART or SPI to the tactical display.

## Non-Claim

This assessment documents architectural alignment with SOSA/MORA principles.
No formal SOSA conformance testing or MORA certification has been performed.
Formal conformance requires testing against the SOSA Technical Standard
and MORA Interface Control Documents, which is deferred to Phase I
integration on a specific target platform.

## Contact

Licensing and integration: licensing@invariantforge.net
