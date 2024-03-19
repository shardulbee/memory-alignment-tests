// Each cache line is typically 64 bytes on x86 architectures.
// We will create an array of u8s with 3 cache lines worth of data, which is 192 bytes.
const CACHE_LINE_SIZE: usize = 64;
const CACHE_LINES: usize = 3;
const DATA_SIZE: usize = CACHE_LINE_SIZE * CACHE_LINES;

// now create an unaligned DATA_SIZE
// this will be used to compare the performance of the aligned and unaligned data
const UNALIGNED_DATA_SIZE: usize = 7;

// make sure we fill an array of each struct with enough data to fill up the cache fully
// we are using an M3 max which has something like 32MiB L2 cache
const CACHE_SIZE: usize = 1_572_864;

struct Aligned {
    data: [u8; DATA_SIZE],
}

impl Aligned {
    fn new() -> Self {
        Aligned {
            // Initialize the array with zeros. This will occupy 192 bytes (3 cache lines of 64 bytes each).
            data: [0; DATA_SIZE],
        }
    }
}

// now create a struct that is explicitly tagged as aligned
#[repr(align(64))]
struct Aligned64 {
    data: [u8; DATA_SIZE],
}
impl Aligned64 {
    fn new() -> Self {
        Aligned64 {
            // Initialize the array with zeros. This will occupy 192 bytes (3 cache lines of 64 bytes each).
            data: [0; DATA_SIZE],
        }
    }
}

// now create a struct of the same size that is explicitly not aligned to cache lines
// this should maximize cache misses and loading whole pages into memory
#[repr(align(64))]
struct Unaligned {
    data: [u8; UNALIGNED_DATA_SIZE],
}
impl Unaligned {
    fn new() -> Self {
        Unaligned {
            data: [0; UNALIGNED_DATA_SIZE],
        }
    }
}

fn run_unaligned() {
    eprintln!("Starting benchmark for unaligned...");
    // create a vector of unaligned data that will fill up the cache
    // then iterate over the vector, calling sum() on each element
    let mut arr = Vec::with_capacity(CACHE_SIZE / UNALIGNED_DATA_SIZE);
    for _ in 0..CACHE_SIZE / UNALIGNED_DATA_SIZE {
        arr.push(Unaligned::new());
    }
    for _ in 0..100 {
        arr.iter_mut().for_each(|x| {
            for i in 0..UNALIGNED_DATA_SIZE {
                x.data[i] = (x.data[i] + 1) % 255;
            }
        });
    }
    eprintln!("Benchmark done.");
}

fn run_explicitly_aligned() {
    eprintln!("Starting benchmark for explicitly aligned...");
    let mut arr = Vec::with_capacity(CACHE_SIZE / DATA_SIZE);
    for _ in 0..CACHE_SIZE / DATA_SIZE {
        arr.push(Aligned64::new());
    }
    for _ in 0..100 {
        arr.iter_mut().for_each(|x| {
            for i in 0..DATA_SIZE {
                x.data[i] = (x.data[i] + 1) % 255;
            }
        });
    }
    eprintln!("Benchmark done.");
}

fn run_aligned() {
    eprintln!("Starting benchmark for not explicitly aligned...");
    let mut arr = Vec::with_capacity(CACHE_SIZE / DATA_SIZE);
    for _ in 0..CACHE_SIZE / DATA_SIZE {
        arr.push(Aligned::new());
    }
    for _ in 0..100 {
        arr.iter_mut().for_each(|x| {
            for i in 0..DATA_SIZE {
                x.data[i] = (x.data[i] + 1) % 255;
            }
        });
    }
    eprintln!("Benchmark done.");
}

enum AlignmentType {
    Explicit,
    Implicit,
    None,
}

fn get_alignment_type_from_str(alignment_str: &str) -> Result<AlignmentType, String> {
    match alignment_str {
        "explicit" => Ok(AlignmentType::Explicit),
        "implicit" => Ok(AlignmentType::Implicit),
        "none" => Ok(AlignmentType::None),
        _ => Err(format!("Invalid alignment type: {}", alignment_str)),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <alignment_type>", args[0]);
        std::process::exit(1);
    }
    let alignment_type_str = &args[1];
    match get_alignment_type_from_str(alignment_type_str) {
        Ok(AlignmentType::Explicit) => run_explicitly_aligned(),
        Ok(AlignmentType::Implicit) => run_aligned(),
        Ok(AlignmentType::None) => run_unaligned(),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
// cargo b --release && hyperfine --min-runs 100 --warmup 3 --command-name none './target/release/my-project none' --command-name implicit './target/release/my-project implicit' --command-name explicit './target/release/my-project explicit'
