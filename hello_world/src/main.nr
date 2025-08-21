// todo add floating point math
fn main(tier: u32, bonus: pub u32) -> pub u32 {
    tier * bonus
}

#[test]
fn test_main() {
    let bonus: u32 = 10;
    assert(main(1, bonus) == 10);
    assert(main(2, bonus) == 20);
    assert(main(3, bonus) == 30);
}
