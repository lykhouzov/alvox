use crate::{model::ModelVertex, vertex::Vertex, Position};

pub struct Faces {
    back: [Vertex; 4],
    front: [Vertex; 4],
    left: [Vertex; 4],
    right: [Vertex; 4],
    top: [Vertex; 4],
    bottom: [Vertex; 4],
}

#[derive(Debug)]
pub struct Voxel {
    position: Position,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Voxel {
    pub const VERTICES: &'static [ModelVertex; 8] = &[
        ModelVertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
        }, // A
        ModelVertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
        }, // B
        ModelVertex {
            position: [1.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
        }, // C
        ModelVertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
        }, // D
        ModelVertex {
            position: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
        }, // E
        ModelVertex {
            position: [0.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
        }, // F
        ModelVertex {
            position: [1.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
        }, // G
        ModelVertex {
            position: [1.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 0.0],
        }, // H
    ];
    #[rustfmt::skip]
    pub const INDICES: &'static [u32] = &[
        0, 3, 1, 1, 3, 2,
        5, 6, 4, 4, 6, 7,
        3, 7, 2, 2, 7, 6,
        1, 5, 0, 0, 5, 4,
        4, 7, 0, 0, 7, 3,
        1, 2, 5, 5, 2, 6
    ];

    pub const FACE_BACK: [ModelVertex; 4] = [
        ModelVertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            normal: [0.0, 0.0, -1.0],
            tangent: [1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
        },
        ModelVertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            normal: [0.0, 0.0, -1.0],
            tangent: [1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
        },
        ModelVertex {
            position: [1.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, -1.0],
            tangent: [1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
        },
        ModelVertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            normal: [0.0, 0.0, -1.0],
            tangent: [1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
        },
    ];
    pub const FACE_FRONT: [ModelVertex; 4] = [
        ModelVertex {
            position: [1.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tangent: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
        },
        ModelVertex {
            position: [1.0, 1.0, 1.0],
            tex_coords: [1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            tangent: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
        },
        ModelVertex {
            position: [0.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
            tangent: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
        },
        ModelVertex {
            position: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 1.0],
            normal: [0.0, 0.0, 1.0],
            tangent: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 1.0, 0.0],
        },
    ];
    pub const FACE_TOP: [ModelVertex; 4] = [
        ModelVertex {
            position: [1.0, 1.0, 1.0],
            tex_coords: [1.0, 1.0],
            normal: [0.0, 1.0, 0.0],
            tangent: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, -1.0],
        },
        ModelVertex {
            position: [1.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            tangent: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, -1.0],
        },
        ModelVertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            tangent: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, -1.0],
        },
        ModelVertex {
            position: [0.0, 1.0, 1.0],
            tex_coords: [0.0, 1.0],
            normal: [0.0, 1.0, 0.0],
            tangent: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, -1.0],
        },
    ];
    pub const FACE_BOTTOM: [ModelVertex; 4] = [
        ModelVertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            normal: [0.0, -1.0, 0.0],
            tangent: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 1.0],
        },
        ModelVertex {
            position: [1.0, 0.0, 1.0],
            tex_coords: [1.0, 0.0],
            normal: [0.0, -1.0, 0.0],
            tangent: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 1.0],
        },
        ModelVertex {
            position: [0.0, 0.0, 1.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, -1.0, 0.0],
            tangent: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 1.0],
        },
        ModelVertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            normal: [0.0, -1.0, 0.0],
            tangent: [-1.0, 0.0, 0.0],
            bitangent: [0.0, 0.0, 1.0],
        },
    ];
    pub const FACE_LEFT: [ModelVertex; 4] = [
        ModelVertex {
            position: [0.0, 0.0, 1.0],
            tex_coords: [1.0, 1.0],
            normal: [-1.0, 0.0, 0.0],
            tangent: [0.0, 0.0, -1.0],
            bitangent: [0.0, 1.0, 0.0],
        },
        ModelVertex {
            position: [0.0, 1.0, 1.0],
            tex_coords: [1.0, 0.0],
            normal: [-1.0, 0.0, 0.0],
            tangent: [0.0, 0.0, -1.0],
            bitangent: [0.0, 1.0, 0.0],
        },
        ModelVertex {
            position: [0.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [-1.0, 0.0, 0.0],
            tangent: [0.0, 0.0, -1.0],
            bitangent: [0.0, 1.0, 0.0],
        },
        ModelVertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [0.0, 1.0],
            normal: [-1.0, 0.0, 0.0],
            tangent: [0.0, 0.0, -1.0],
            bitangent: [0.0, 1.0, 0.0],
        },
    ];
    pub const FACE_RIGHT: [ModelVertex; 4] = [
        ModelVertex {
            position: [1.0, 0.0, 0.0],
            tex_coords: [1.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 1.0],
            bitangent: [0.0, 1.0, 0.0],
        },
        ModelVertex {
            position: [1.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
            normal: [1.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 1.0],
            bitangent: [0.0, 1.0, 0.0],
        },
        ModelVertex {
            position: [1.0, 1.0, 1.0],
            tex_coords: [0.0, 0.0],
            normal: [1.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 1.0],
            bitangent: [0.0, 1.0, 0.0],
        },
        ModelVertex {
            position: [1.0, 0.0, 1.0],
            tex_coords: [0.0, 1.0],
            normal: [1.0, 0.0, 0.0],
            tangent: [0.0, 0.0, 1.0],
            bitangent: [0.0, 1.0, 0.0],
        },
    ];
    pub const FACES: &'static [[ModelVertex; 4]; 6] = &[
        Self::FACE_BACK,
        Self::FACE_FRONT,
        Self::FACE_TOP,
        Self::FACE_BOTTOM,
        Self::FACE_LEFT,
        Self::FACE_RIGHT,
    ];
    #[rustfmt::skip]
    #[allow(dead_code)]
    pub const INDICES_FACES: &'static [[usize; 6]; 6] = &[
        [0, 3, 1, 1, 3, 2],
        [5, 6, 4, 4, 6, 7],
        [3, 7, 2, 2, 7, 6],
        [1, 5, 0, 0, 5, 4],
        [4, 7, 0, 0, 7, 3],
        [1, 2, 5, 5, 2, 6]
    ];

    #[rustfmt::skip]
    pub const UVS: &'static [[f32; 2]; 4] = &[
        [0.0, 1.0],
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
    ];

    #[allow(dead_code)]
    pub const FACE_CHECK: &'static [Position; 6] = &[
        Position::new(0.0, 0.0, -1.0), // Back
        Position::new(0.0, 0.0, 1.0),  // Front
        Position::new(0.0, 1.0, 0.0),  // Top
        Position::new(0.0, -1.0, 0.0), // Bottom
        Position::new(-1.0, 0.0, 0.0), // Left
        Position::new(1.0, 0.0, 0.0),  // Right
    ];
}

pub struct Color {}

#[allow(dead_code)]
impl Color {
    const RED: [f32; 3] = [1.0, 0.0, 0.0];
    const BLACK: [f32; 3] = [0.0, 0.0, 0.0];
    const WIGHT: [f32; 3] = [1.0, 1.0, 1.0];
    const PINK: [f32; 3] = [0.5, 0.0, 0.5];
    const GREY: [f32; 3] = [0.5, 0.5, 0.5];
}

pub const CUBE: &'static [ModelVertex; 24] = &[
    ModelVertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [0.875, 0.5],
        normal: [0.0, 1.0, 0.0],
        tangent: [-1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [1.0, 1.0, 1.0],
        tex_coords: [0.625, 0.75],
        normal: [0.0, 1.0, 0.0],
        tangent: [-1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [0.625, 0.5],
        normal: [0.0, 1.0, 0.0],
        tangent: [-1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [1.0, 1.0, 1.0],
        tex_coords: [0.625, 0.75],
        normal: [0.0, 0.0, 1.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [1.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [0.0, 0.0, 1.0],
        tex_coords: [0.375, 1.0],
        normal: [0.0, 0.0, 1.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [1.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [1.0, 0.0, 1.0],
        tex_coords: [0.375, 0.75],
        normal: [0.0, 0.0, 1.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [1.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [0.0, 1.0, 1.0],
        tex_coords: [0.625, 0.0],
        normal: [-1.0, 0.0, 0.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [0.0, 0.0, 1.0],
    },
    ModelVertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.375, 0.25],
        normal: [-1.0, 0.0, 0.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [0.0, 0.0, 1.0],
    },
    ModelVertex {
        position: [0.0, 0.0, 1.0],
        tex_coords: [0.375, 0.0],
        normal: [-1.0, 0.0, 0.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [0.0, 0.0, 1.0],
    },
    ModelVertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [0.375, 0.5],
        normal: [0.0, -1.0, 0.0],
        tangent: [1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [0.0, 0.0, 1.0],
        tex_coords: [0.125, 0.75],
        normal: [0.0, -1.0, 0.0],
        tangent: [1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.125, 0.5],
        normal: [0.0, -1.0, 0.0],
        tangent: [1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [0.625, 0.5],
        normal: [1.0, 0.0, 0.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [1.0, 0.0, 1.0],
        tex_coords: [0.375, 0.75],
        normal: [1.0, 0.0, 0.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [0.375, 0.5],
        normal: [1.0, 0.0, 0.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [0.625, 0.25],
        normal: [0.0, 0.0, -1.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [-1.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [0.375, 0.5],
        normal: [0.0, 0.0, -1.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [-1.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.375, 0.25],
        normal: [0.0, 0.0, -1.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [-1.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [0.0, 1.0, 1.0],
        tex_coords: [0.875, 0.75],
        normal: [0.0, 1.0, 0.0],
        tangent: [-1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [0.0, 1.0, 1.0],
        tex_coords: [0.625, 1.0],
        normal: [0.0, 0.0, 1.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [1.0, 0.0, 0.0],
    },
    ModelVertex {
        position: [0.0, 1.0, 0.0],
        tex_coords: [0.625, 0.25],
        normal: [-1.0, 0.0, 0.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [0.0, 0.0, 1.0],
    },
    ModelVertex {
        position: [1.0, 0.0, 1.0],
        tex_coords: [0.375, 0.75],
        normal: [0.0, -1.0, 0.0],
        tangent: [1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [1.0, 1.0, 1.0],
        tex_coords: [0.625, 0.75],
        normal: [1.0, 0.0, 0.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [0.625, 0.5],
        normal: [0.0, 0.0, -1.0],
        tangent: [0.0, 1.0, 0.0],
        bitangent: [-1.0, 0.0, 0.0],
    },
];

#[rustfmt::skip]
pub const CUBE_INDICES: &'static [u32] = &[
    0, 1, 2, 
    3, 4, 5, 
    6, 7, 8, 
    9, 10, 11, 
    12, 13, 14, 
    15, 16, 17, 
    
    0, 18, 1, 
    3, 19, 4, 
    6, 20, 7, 
    9, 21, 10, 
    12, 22, 13, 
    15, 23, 16,
];

pub const PLANE: &'static [ModelVertex] = &[
    ModelVertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
        tangent: [-1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [1.0, 0.0, 1.0],
        tex_coords: [1.0, 0.0],
        normal: [0.0, 1.0, 0.0],
        tangent: [-1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
        normal: [0.0, 1.0, 0.0],
        tangent: [-1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    ModelVertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 1.0],
        normal: [0.0, 1.0, 0.0],
        tangent: [-1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, -1.0],
    },
    // BOTTOM
    ModelVertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
        tangent: [-1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, 1.0],
    },
    ModelVertex {
        position: [1.0, 0.0, 1.0],
        tex_coords: [1.0, 0.0],
        normal: [0.0, -1.0, 0.0],
        tangent: [-1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, 1.0],
    },
    ModelVertex {
        position: [0.0, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
        normal: [0.0, -1.0, 0.0],
        tangent: [-1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, 1.0],
    },
    ModelVertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 1.0],
        normal: [0.0, -1.0, 0.0],
        tangent: [-1.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, 1.0],
    },
];

pub const PLANE_INDICES: &'static [u32] = &[3, 2, 1, 3, 1, 0, 7, 5, 6, 7, 4, 5];
