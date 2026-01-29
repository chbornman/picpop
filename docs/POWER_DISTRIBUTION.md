# PicPop Power Distribution Plan

Single power supply feeding an intermediate power board that provides 12V for the monitor and 5V for the Radxa and camera.

---

## OPTION 1: USB-C PD Power Supply

Uses a USB-C Power Delivery brick with a PD trigger to negotiate 12V.

### System Diagram (Option 1)

```
                         OPTION 1: USB-C PD POWER SUPPLY
                              PicPop Photo Booth

┌─────────────────────────────────────────────────────────────────────────────┐
│                           USB-C POWER SUPPLY                                │
│                         (e.g., 65W+ USB-PD Brick)                           │
│                                                                             │
│    AC Mains ───► [USB-C PD Power Adapter] ───► USB-C Cable Out             │
│                                                                             │
│                  Requirements: Must support 12V/3A+ PD profile              │
└─────────────────────────────────────────────────────────────┬───────────────┘
                                                              │
                                                              │ USB-C Cable
                                                              │ (PD Capable)
                                                              ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        POWER DISTRIBUTION BOARD                             │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                      USB-C PD INPUT STAGE                             │  │
│  │                                                                       │  │
│  │   USB-C      ┌──────────────────┐                                    │  │
│  │   Female ───►│  USB-PD Trigger  │───► 12V Rail                       │  │
│  │   Connector  │  (IP2721/HUSB238)│     (Negotiated)                   │  │
│  │              │  Requests 12V    │                                    │  │
│  │              └──────────────────┘                                    │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                      │                                      │
│                                      │ 12V Rail                             │
│                                      │                                      │
│           ┌──────────────────────────┼──────────────────────────┐           │
│           │                          │                          │           │
│           ▼                          ▼                          ▼           │
│  ┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐   │
│  │  12V OUTPUT     │       │  5V REGULATOR   │       │  (OPTIONAL)     │   │
│  │                 │       │                 │       │  Reserved       │   │
│  │  Barrel Jack    │       │  Buck Converter │       │                 │   │
│  │  5.5x2.1mm or   │       │  (12V → 5V)     │       │                 │   │
│  │  5.5x2.5mm      │       │  (e.g., MP1584) │       │                 │   │
│  │                 │       │  3A+ Output     │       │                 │   │
│  └────────┬────────┘       └────────┬────────┘       └─────────────────┘   │
│           │                         │                                       │
│           │                         │ 5V Rail                               │
│           │                         │                                       │
│           │              ┌──────────┴──────────┐                            │
│           │              │                     │                            │
│           │              ▼                     ▼                            │
│           │     ┌─────────────────┐   ┌─────────────────┐                  │
│           │     │  USB-A OUTPUT   │   │  USB-A OUTPUT   │                  │
│           │     │  (Radxa 3W)     │   │  (Dummy Battery)│                  │
│           │     │                 │   │                 │                  │
│           │     │  5V @ 2A        │   │  5V @ 1.5A      │                  │
│           │     └────────┬────────┘   └────────┬────────┘                  │
│           │              │                     │                            │
└───────────┼──────────────┼─────────────────────┼────────────────────────────┘
            │              │                     │
            ▼              ▼                     ▼
    ┌───────────────┐  ┌───────────────┐  ┌───────────────────────┐
    │   27" MONITOR │  │  RADXA ZERO   │  │  CAMERA DUMMY         │
    │               │  │     3W        │  │  BATTERY              │
    │  12V @ 2-3A   │  │               │  │                       │
    │  via Barrel   │  │  5V via USB   │  │  USB-A to Dummy       │
    │  Jack         │  │  (Type-C)     │  │  Battery Coupler      │
    │               │  │               │  │  (Sony NP-FZ100 etc)  │
    └───────────────┘  └───────────────┘  └───────────────────────┘
```

### Wiring Summary (Option 1)

```
[USB-C PD Brick] ──USB-C──► [PD Trigger] ──12V──┬──► [Barrel Jack] ──► Monitor
                                                │
                                                └──► [Buck 12→5V] ──5V──┬──► USB-A ──► Radxa
                                                                        │
                                                                        └──► USB-A ──► Dummy Bat ──► Camera
```

### USB-C PD Trigger Options

| Module  | Price | Notes                               |
| ------- | ----- | ----------------------------------- |
| IP2721  | $2-5  | Simple fixed 12V trigger            |
| HUSB238 | $3-6  | Programmable, I2C control           |
| CH224K  | $2-4  | Popular, configurable via resistors |

### Option 1 Pros/Cons

| Pros                            | Cons                               |
| ------------------------------- | ---------------------------------- |
| Compact, portable power brick   | Requires PD trigger module         |
| USB-C is universal/modern       | Not all bricks support 12V profile |
| Easy to find replacement bricks | More complex distribution board    |
| Can double as laptop charger    | PD negotiation adds failure point  |

---

## OPTION 2: 12V DC Power Supply

Uses a traditional 12V DC barrel jack power supply (like a laptop charger).

### System Diagram (Option 2)

```
                         OPTION 2: 12V DC POWER SUPPLY
                              PicPop Photo Booth

┌─────────────────────────────────────────────────────────────────────────────┐
│                          12V DC POWER SUPPLY                                │
│                      (e.g., 12V/5A Barrel Jack PSU)                         │
│                                                                             │
│    AC Mains ───► [12V DC Adapter] ───► Barrel Jack Cable Out               │
│                                                                             │
│                  Requirements: 12V @ 5A (60W) minimum                       │
└─────────────────────────────────────────────────────────────┬───────────────┘
                                                              │
                                                              │ Barrel Jack
                                                              │ (5.5x2.1mm)
                                                              ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        POWER DISTRIBUTION BOARD                             │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                      DC INPUT STAGE                                   │  │
│  │                                                                       │  │
│  │   Barrel      ┌──────────────────┐                                   │  │
│  │   Jack    ───►│  Input Protection│───► 12V Rail                      │  │
│  │   Female      │  (Fuse + Diode)  │     (Direct)                      │  │
│  │   5.5x2.1mm   │                  │                                   │  │
│  │               └──────────────────┘                                   │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                      │                                      │
│                                      │ 12V Rail                             │
│                                      │                                      │
│           ┌──────────────────────────┼──────────────────────────┐           │
│           │                          │                          │           │
│           ▼                          ▼                          ▼           │
│  ┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐   │
│  │  12V OUTPUT     │       │  5V REGULATOR   │       │  (OPTIONAL)     │   │
│  │                 │       │                 │       │  Reserved       │   │
│  │  Barrel Jack    │       │  Buck Converter │       │                 │   │
│  │  5.5x2.1mm or   │       │  (12V → 5V)     │       │                 │   │
│  │  5.5x2.5mm      │       │  (e.g., MP1584) │       │                 │   │
│  │                 │       │  3A+ Output     │       │                 │   │
│  └────────┬────────┘       └────────┬────────┘       └─────────────────┘   │
│           │                         │                                       │
│           │                         │ 5V Rail                               │
│           │                         │                                       │
│           │              ┌──────────┴──────────┐                            │
│           │              │                     │                            │
│           │              ▼                     ▼                            │
│           │     ┌─────────────────┐   ┌─────────────────┐                  │
│           │     │  USB-A OUTPUT   │   │  USB-A OUTPUT   │                  │
│           │     │  (Radxa 3W)     │   │  (Dummy Battery)│                  │
│           │     │                 │   │                 │                  │
│           │     │  5V @ 2A        │   │  5V @ 1.5A      │                  │
│           │     └────────┬────────┘   └────────┬────────┘                  │
│           │              │                     │                            │
└───────────┼──────────────┼─────────────────────┼────────────────────────────┘
            │              │                     │
            ▼              ▼                     ▼
    ┌───────────────┐  ┌───────────────┐  ┌───────────────────────┐
    │   27" MONITOR │  │  RADXA ZERO   │  │  CAMERA DUMMY         │
    │               │  │     3W        │  │  BATTERY              │
    │  12V @ 2-3A   │  │               │  │                       │
    │  via Barrel   │  │  5V via USB   │  │  USB-A to Dummy       │
    │  Jack         │  │  (Type-C)     │  │  Battery Coupler      │
    │               │  │               │  │  (Sony NP-FZ100 etc)  │
    └───────────────┘  └───────────────┘  └───────────────────────┘
```

### Wiring Summary (Option 2)

```
[12V DC PSU] ──Barrel──► [Fuse/Protection] ──12V──┬──► [Barrel Jack] ──► Monitor
                                                  │
                                                  └──► [Buck 12→5V] ──5V──┬──► USB-A ──► Radxa
                                                                          │
                                                                          └──► USB-A ──► Dummy Bat ──► Camera
```

### Option 2 Pros/Cons

| Pros                             | Cons                       |
| -------------------------------- | -------------------------- |
| Simple, no PD negotiation needed | Bulkier power supply       |
| Cheap and widely available       | Less portable              |
| Direct 12V, fewer failure points | Need specific voltage PSU  |
| Easy to find high-current units  | Extra cable/brick to carry |

---

## Shared Components (Both Options)

### Buck Converter (12V to 5V)

| Module  | Price | Max Current | Notes                             |
| ------- | ----- | ----------- | --------------------------------- |
| MP1584  | $1-3  | 3A          | Compact, good efficiency          |
| LM2596  | $2-4  | 3A          | Adjustable output                 |
| Mini360 | $1-2  | 1.8A        | Very compact (single device only) |

### Barrel Jack Connector (Monitor Output)

- **Check your monitor's barrel jack size!**
- Common sizes: 5.5x2.1mm or 5.5x2.5mm
- Verify polarity: Usually center-positive

---

## Power Budget

| Component              | Voltage | Current   | Power      |
| ---------------------- | ------- | --------- | ---------- |
| 27" Monitor            | 12V     | 2-3A      | 24-36W     |
| Radxa ZERO 3W          | 5V      | 2A (peak) | 10W        |
| Dummy Battery (Camera) | 5V      | 1.5A      | 7.5W       |
| **Total**              |         |           | **42-54W** |

**Recommendation:**

- Option 1: Use a 65W+ USB-PD power supply (must support 12V profile)
- Option 2: Use a 12V/5A (60W) DC adapter minimum

---

## Key Considerations

1. **Option 1 - USB-PD Negotiation**: The PD trigger module must successfully negotiate 12V. Verify your brick supports the 12V PD profile (not all do - many jump from 9V to 15V or 20V).

2. **Option 2 - Voltage Tolerance**: Cheap 12V adapters may output 12.5-13V. This is fine for monitors and buck converters but verify your monitor's tolerance.

3. **Barrel Jack Sizing**: Measure your monitor's barrel jack carefully. 5.5x2.1mm and 5.5x2.5mm look nearly identical but won't interchange properly.

4. **5V Rail Current**: The buck converter handles combined load of Radxa (~2A peak) + dummy battery (~1.5A). Use a converter rated for 4-5A minimum.

5. **Heat Dissipation**: The buck converter will dissipate ~3.5W as heat at full load. Ensure adequate ventilation or add a small heatsink.

6. **Input Protection (Option 2)**: Add a fuse (5A) and reverse polarity diode to protect against wrong adapter or shorts.

---

## Recommendation

**Option 2 (12V DC) is simpler and more reliable** for a stationary photo booth:

- No PD negotiation complexity
- Cheaper overall
- Easier to troubleshoot
- More headroom on current capacity

**Option 1 (USB-C PD) is better if:**

- Portability is important
- You want to use one charger for multiple devices
- You already have a compatible USB-PD brick
