#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureFlags(pub u32);

impl TextureFlags {
    pub const POINTSAMPLE: Self = Self(0x0001);
    pub const TRILINEAR: Self = Self(0x0002);
    pub const CLAMPS: Self = Self(0x0004);
    pub const CLAMPT: Self = Self(0x0008);
    pub const ANISOTROPIC: Self = Self(0x0010);
    pub const HINT_DXT5: Self = Self(0x0020);
    pub const PWL_CORRECTED: Self = Self(0x0040);
    pub const NORMAL: Self = Self(0x0080);
    pub const NO_MIP: Self = Self(0x0100);
    pub const NO_LOD: Self = Self(0x0200);
    pub const ALL_MIPS: Self = Self(0x0400);
    pub const PROCEDURAL: Self = Self(0x0800);
    pub const ONEBITALPHA: Self = Self(0x1000);
    pub const EIGHTBITALPHA: Self = Self(0x2000);
    pub const ENVMAP: Self = Self(0x4000);
    pub const RENDER_TARGET: Self = Self(0x8000);
    pub const DEPTH_RENDER_TARGET: Self = Self(0x0001_0000);
    pub const NO_DEBUG_OVERRIDE: Self = Self(0x0002_0000);
    pub const SINGLE_COPY: Self = Self(0x0004_0000);
    pub const SRGB: Self = Self(0x0040_0000);

    pub const EMPTY: Self = Self(0);

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn bits(self) -> u32 {
        self.0
    }
}

impl std::ops::BitOr for TextureFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for TextureFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for TextureFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::Not for TextureFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}
