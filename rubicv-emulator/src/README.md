Memory Layout:

0x0000_0000 +------------------------+
            |                       |
            |     Code  (8 KB)      |
            |                       |
0x0002_0000 +------------------------+
            |    Scratch (256 B)    |
            +------------------------+
            |                       |
            |         Heap          |
            |     (Grows up ↑)      |
            |                       |
            +------------------------+
            |         Stack         |
            |    (Grows down ↓)     |
0x0001_0000 +========================+  
            |     Args (256 B)      |
            +------------------------+
            |                       |
            |                       |
            |       RO Slab         |
            |       (~4 MB)         |
            |                       |
            |                       |
0x0040_0000 +------------------------+

Memory Regions:
└── Total Memory: 4 MB (0x0040_0000)
├── RW Region: 64 KB (0x0001_0000)
│   ├── Code:    8 KB (0x0000_2000)
│   ├── Scratch: 256 B
│   ├── Heap:    Growing upward
│   └── Stack:   Growing downward from top of RW
└── RO Region: ~4 MB (0x003F_0000)
├── Args:    256 B
└── RO Slab: Remainder of RO space

Memory Masks:
- RW Mask:     0x0000_FFFF
- Memory Mask: 0x003F_FFFF