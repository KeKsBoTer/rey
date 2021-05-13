use crate::lib::Vertex;

pub const VERTICES: [Vertex; 72] = [
    // Floor
    Vertex {
        position: [552.8, 0.0, 0.0],
    },
    Vertex {
        position: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.0, 0.0, 559.2],
    },
    Vertex {
        position: [549.6, 0.0, 559.2],
    },
    Vertex {
        position: [130.0, 0.0, 65.0],
    },
    Vertex {
        position: [82.0, 0.0, 225.0],
    },
    Vertex {
        position: [240.0, 0.0, 272.0],
    },
    Vertex {
        position: [290.0, 0.0, 114.0],
    },
    Vertex {
        position: [423.0, 0.0, 247.0],
    },
    Vertex {
        position: [265.0, 0.0, 296.0],
    },
    Vertex {
        position: [314.0, 0.0, 456.0],
    },
    Vertex {
        position: [472.0, 0.0, 406.0],
    },
    // Ceiling
    Vertex {
        position: [556.0, 548.8, 0.0],
    },
    Vertex {
        position: [556.0, 548.8, 559.2],
    },
    Vertex {
        position: [0.0, 548.8, 559.2],
    },
    Vertex {
        position: [0.0, 548.8, 0.0],
    },
    Vertex {
        position: [343.0, 548.8, 227.0],
    },
    Vertex {
        position: [343.0, 548.8, 332.0],
    },
    Vertex {
        position: [213.0, 548.8, 332.0],
    },
    Vertex {
        position: [213.0, 548.8, 227.0],
    },
    // Back wall
    Vertex {
        position: [549.6, 0.0, 559.2],
    },
    Vertex {
        position: [0.0, 0.0, 559.2],
    },
    Vertex {
        position: [0.0, 548.8, 559.2],
    },
    Vertex {
        position: [556.0, 548.8, 559.2],
    },
    // Right wall
    Vertex {
        position: [0.0, 0.0, 559.2],
    },
    Vertex {
        position: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.0, 548.8, 0.0],
    },
    Vertex {
        position: [0.0, 548.8, 559.2],
    },
    // Left wall
    Vertex {
        position: [552.8, 0.0, 0.0],
    },
    Vertex {
        position: [549.6, 0.0, 559.2],
    },
    Vertex {
        position: [556.0, 548.8, 559.2],
    },
    Vertex {
        position: [556.0, 548.8, 0.0],
    },
    // Short block
    Vertex {
        position: [130.0, 165.0, 65.0],
    },
    Vertex {
        position: [82.0, 165.0, 225.0],
    },
    Vertex {
        position: [240.0, 165.0, 272.0],
    },
    Vertex {
        position: [290.0, 165.0, 114.0],
    },
    Vertex {
        position: [290.0, 0.0, 114.0],
    },
    Vertex {
        position: [290.0, 165.0, 114.0],
    },
    Vertex {
        position: [240.0, 165.0, 272.0],
    },
    Vertex {
        position: [240.0, 0.0, 272.0],
    },
    Vertex {
        position: [130.0, 0.0, 65.0],
    },
    Vertex {
        position: [130.0, 165.0, 65.0],
    },
    Vertex {
        position: [290.0, 165.0, 114.0],
    },
    Vertex {
        position: [290.0, 0.0, 114.0],
    },
    Vertex {
        position: [82.0, 0.0, 225.0],
    },
    Vertex {
        position: [82.0, 165.0, 225.0],
    },
    Vertex {
        position: [130.0, 165.0, 65.0],
    },
    Vertex {
        position: [130.0, 0.0, 65.0],
    },
    Vertex {
        position: [240.0, 0.0, 272.0],
    },
    Vertex {
        position: [240.0, 165.0, 272.0],
    },
    Vertex {
        position: [82.0, 165.0, 225.0],
    },
    Vertex {
        position: [82.0, 0.0, 225.0],
    },
    // Tall block
    Vertex {
        position: [423.0, 330.0, 247.0],
    },
    Vertex {
        position: [265.0, 330.0, 296.0],
    },
    Vertex {
        position: [314.0, 330.0, 456.0],
    },
    Vertex {
        position: [472.0, 330.0, 406.0],
    },
    Vertex {
        position: [423.0, 0.0, 247.0],
    },
    Vertex {
        position: [423.0, 330.0, 247.0],
    },
    Vertex {
        position: [472.0, 330.0, 406.0],
    },
    Vertex {
        position: [472.0, 0.0, 406.0],
    },
    Vertex {
        position: [472.0, 0.0, 406.0],
    },
    Vertex {
        position: [472.0, 330.0, 406.0],
    },
    Vertex {
        position: [314.0, 330.0, 456.0],
    },
    Vertex {
        position: [314.0, 0.0, 456.0],
    },
    Vertex {
        position: [314.0, 0.0, 456.0],
    },
    Vertex {
        position: [314.0, 330.0, 456.0],
    },
    Vertex {
        position: [265.0, 330.0, 296.0],
    },
    Vertex {
        position: [265.0, 0.0, 296.0],
    },
    Vertex {
        position: [265.0, 0.0, 296.0],
    },
    Vertex {
        position: [265.0, 330.0, 296.0],
    },
    Vertex {
        position: [423.0, 330.0, 247.0],
    },
    Vertex {
        position: [423.0, 0.0, 247.0],
    },
];

pub const FACES: [[u32; 3]; 36] = [
    // Floor
    [0, 1, 2],
    [2, 3, 0],
    [4, 5, 6],
    [6, 7, 4],
    [8, 9, 10],
    [10, 11, 8],
    // Ceiling
    [12, 13, 14],
    [14, 15, 12],
    [16, 17, 18],
    [18, 19, 16],
    // Back wall
    [20, 21, 22],
    [22, 23, 20],
    // Right wall
    [24, 25, 26],
    [26, 27, 24],
    // Left wall
    [28, 29, 30],
    [30, 31, 28],
    // Short block
    [32, 33, 34],
    [34, 35, 32],
    [36, 37, 38],
    [38, 39, 36],
    [40, 41, 42],
    [42, 43, 40],
    [44, 45, 46],
    [46, 47, 44],
    [48, 49, 50],
    [50, 51, 48],
    // Tall block
    [52, 53, 54],
    [54, 55, 52],
    [56, 57, 58],
    [58, 59, 56],
    [60, 61, 62],
    [62, 63, 60],
    [64, 65, 66],
    [66, 67, 64],
    [68, 69, 70],
    [70, 71, 68],
];
