#![feature(generic_const_exprs)]

pub use proc::vecn;

pub const fn new_array<const Size: usize>() -> [usize; Size] {
    let mut arr = [0; Size];
    let mut i = 0;
    while i < Size {
        arr[i] = i;
        i += 1;
    }
    arr
}

pub const fn new_array_2<const Size: usize>() -> [usize; Size]
where
    [(); Size + 1]:,
{
    let arr = [0; Size];
    fill_array::<Size, 1>(arr)
}

pub const fn fill_array<const Size: usize, const Index: usize>(
    mut arr: [usize; Size],
) -> [usize; Size]
where
    [(); Index + 1]:,
{
    arr[Index] = Index;
    let arr = arr;
    if Index > Size {
        fill_array::<Size, { Index + 1 }>(arr)
    } else {
        arr
    }
}
