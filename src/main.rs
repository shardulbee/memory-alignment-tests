// #![feature(iter_array_chunks)]

// a big enough number that we should fill up the cache
const CACHE_SIZE: usize = 1_572_864;

fn run_v1(args: &[String]) {
    // Each cache line is typically 64 bytes on x86 architectures.
    // We will create an array of u8s with 3 cache lines worth of data, which is 192 bytes.
    const CACHE_LINE_SIZE: usize = 64;
    const CACHE_LINES: usize = 3;
    const ALIGNED_DATA_SIZE: usize = CACHE_LINE_SIZE * CACHE_LINES;

    // define an unaligned DATA_SIZE
    // this will be used to compare the performance of the aligned and unaligned data
    const UNALIGNED_DATA_SIZE: usize = 7;

    // we'll loop through the vector multiple times just to make sure we're getting consistent results
    const NUM_ITERATIONS: usize = 100;

    // 64 * 3 = 192 bytes = 3 cache lines per Aligned
    struct Aligned([u8; ALIGNED_DATA_SIZE]);
    impl Aligned {
        fn new() -> Self {
            Aligned([0; ALIGNED_DATA_SIZE])
        }
        fn cpu_intensive_op(&mut self) {
            for i in 0..self.0.len() {
                self.0[i] = self.0[i].wrapping_add(1);
            }
        }
    }

    // now create a struct that is explicitly tagged as aligned
    // 64 * 3 = 192 bytes = 3 cache lines per Aligned64
    #[repr(align(64))]
    struct Aligned64([u8; ALIGNED_DATA_SIZE]);
    impl Aligned64 {
        fn new() -> Self {
            Aligned64([0; ALIGNED_DATA_SIZE])
        }
        fn cpu_intensive_op(&mut self) {
            for i in 0..self.0.len() {
                self.0[i] = self.0[i].wrapping_add(1);
            }
        }
    }

    // now create a struct of the same size that is explicitly not aligned to cache lines
    // this should maximize cache misses and loading whole pages into memory
    // 7 bytes, not aligned
    struct Unaligned([u8; UNALIGNED_DATA_SIZE]);
    impl Unaligned {
        fn new() -> Self {
            Unaligned([0; UNALIGNED_DATA_SIZE])
        }
        fn cpu_intensive_op(&mut self) {
            for i in 0..self.0.len() {
                self.0[i] = self.0[i].wrapping_add(1);
            }
        }
    }

    fn run(alignment_type: AlignmentType) {
        match alignment_type {
            AlignmentType::None => {
                let mut arr: Vec<Unaligned> = (0..CACHE_SIZE / UNALIGNED_DATA_SIZE)
                    .map(|_| Unaligned::new())
                    .collect();
                (0..NUM_ITERATIONS).for_each(|_| {
                    arr.iter_mut().for_each(|x| {
                        x.cpu_intensive_op();
                    });
                });
            }
            AlignmentType::Implicit => {
                let mut arr: Vec<Aligned> = (0..CACHE_SIZE / ALIGNED_DATA_SIZE)
                    .map(|_| Aligned::new())
                    .collect();
                (0..NUM_ITERATIONS).for_each(|_| {
                    arr.iter_mut().for_each(|x| {
                        x.cpu_intensive_op();
                    });
                });
            }
            AlignmentType::Explicit => {
                let mut arr: Vec<Aligned64> = (0..CACHE_SIZE / ALIGNED_DATA_SIZE)
                    .map(|_| Aligned64::new())
                    .collect();
                (0..NUM_ITERATIONS).for_each(|_| {
                    arr.iter_mut().for_each(|x| {
                        x.cpu_intensive_op();
                        // volatile read to prevent the compiler from optimizing out the loop
                        // this is a common trick to prevent the compiler from optimizing out the loop
                        // use read_volatile here
                    });
                });
            }
        }
        eprintln!("Benchmark for {} complete", alignment_type);
    }

    enum AlignmentType {
        Explicit,
        Implicit,
        None,
    }
    impl std::fmt::Display for AlignmentType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                AlignmentType::Explicit => write!(f, "explicit"),
                AlignmentType::Implicit => write!(f, "implicit"),
                AlignmentType::None => write!(f, "none"),
            }
        }
    }

    fn get_alignment_type_from_str(alignment_str: &str) -> Result<AlignmentType, String> {
        match alignment_str {
            "explicit" => Ok(AlignmentType::Explicit),
            "implicit" => Ok(AlignmentType::Implicit),
            "none" => Ok(AlignmentType::None),
            _ => Err(format!("Invalid alignment type: {}", alignment_str)),
        }
    }

    let alignment_type_str = &args[2];
    match get_alignment_type_from_str(alignment_type_str) {
        Ok(alignment_type) => run(alignment_type),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

// Trying another experiment where instead of explicitly creating an array of structs with fixed/known size,
// we accept an integer parameter // telling us how big each element of the vec we should allocate. We fill
// the vec with these elements and then iterate over it while performing a CPU intensive operation. This
// will allow us to compare the performance for different values for the size of the elements in the vec
// to see how it affects performance
fn run_v2(args: &[String]) {
    struct DynamicVecElement(Vec<u8>);
    impl DynamicVecElement {
        fn new(size: usize) -> Self {
            DynamicVecElement(vec![0; size])
        }

        fn cpu_intensive_op(&mut self) {
            for i in 0..self.0.len() {
                let _ = self.0[i].wrapping_add(1);
            }
        }
    }

    let size_of_struct = args[2].parse::<usize>();
    if size_of_struct.is_err() {
        eprintln!("Usage: {} v2 <struct-size: usize>", args[0]);
        std::process::exit(1);
    }
    let size_of_struct = size_of_struct.unwrap();
    if size_of_struct == 0 {
        eprintln!("Size of struct must be greater than 0");
        std::process::exit(1);
    }

    let mut arr = Vec::new();
    for _ in 0..CACHE_SIZE / size_of_struct {
        let dynamic_vec_element = DynamicVecElement::new(size_of_struct);
        println!(
            "address={:?}, size={}",
            dynamic_vec_element.0.as_ptr(),
            dynamic_vec_element.0.len()
        );
        arr.push(dynamic_vec_element);
    }
    for _ in 0..259 {
        arr.iter_mut().for_each(|x| {
            x.cpu_intensive_op();
        });
    }
}

fn run_v3(args: &[String]) {
    let arr = [0u8; CACHE_SIZE];

    let stride_length = args[2].parse::<usize>();
    if stride_length.is_err() {
        eprintln!("Usage: {} v2 <stride length: usize>", args[0]);
        std::process::exit(1);
    }
    let stride_length = stride_length.unwrap();
    if stride_length == 0 {
        eprintln!("Stride length must be greater than 0");
        std::process::exit(1);
    }
    for _ in 0..259 {
        (0..CACHE_SIZE).step_by(stride_length).for_each(|i| {
            let chunk = if i + stride_length <= CACHE_SIZE {
                &arr[i..i + stride_length]
            } else {
                &arr[i..]
            };
            unsafe {
                chunk.as_ptr().read_volatile();
            }
        });
    }
}

// run_v4
fn run_v4(args: &[String]) {
    let stride_length = args[2].parse::<usize>();
    if stride_length.is_err() {
        eprintln!("Usage: {} v2 <stride length: usize>", args[0]);
        std::process::exit(1);
    }
    let stride_length = stride_length.unwrap();
    if stride_length == 0 {
        eprintln!("Stride length must be greater than 0");
        std::process::exit(1);
    }
    const COUNT: usize = 24000000;
    let data = vec![1u8; COUNT * stride_length];

    for _ in 0..1000 {
        for i in 0..COUNT {
            let datum: &u8 = &data[i * stride_length];

            unsafe {
                let _ = std::ptr::read_volatile(datum);
            }
        }
    }
}

// add a runtime flag provided at the command line to specify
// whether to run the v1 tests or the v2 tests
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <version> <version args>", args[0]);
        std::process::exit(1);
    }
    let version: &str = &args[1];
    match version {
        "v1" => run_v1(&args),
        "v2" => run_v2(&args),
        "v3" => run_v3(&args),
        "v4" => run_v4(&args),
        _ => {
            eprintln!("Invalid version: {}", version);
            std::process::exit(1);
        }
    };
}
