use contract_sdk_rust::sim_context;
use contract_sdk_rust::sim_context::SimContext;
use sha2::{Sha256, Digest};
use num_bigint::BigInt;
use num_traits::{Zero, One, Pow};

// 安装合约时会执行此方法
#[no_mangle]
pub extern "C" fn init_contract() {
    let ctx = &mut sim_context::get_sim_context();
    ctx.ok("Init contract success".as_bytes());
}

// 升级合约时会执行此方法
#[no_mangle]
pub extern "C" fn upgrade() {
    let ctx = &mut sim_context::get_sim_context();
    ctx.ok("Upgrade contract success".as_bytes());
}


// 普通计算：累加 1 到 1,000,000
#[no_mangle]
pub extern "C" fn normalCal() {
    let ctx = &mut sim_context::get_sim_context();

    let mut result: i64 = 0;
    for i in 0..1000000 {
        result += i as i64;
    }
    let response = format!("success normalCal: {}", result);
    ctx.ok(response.as_bytes());
}

// 哈希计算：执行 100000 次 SHA256
#[no_mangle]
pub extern "C" fn hashCal() {
    let ctx = &mut sim_context::get_sim_context();

    let input = "ChainMaker Performance Test";
    let mut hash_result = [0u8; 32];

    for _ in 0..10000 {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let result = hasher.finalize();
        hash_result.copy_from_slice(&result);
    }

    let hex_str = hash_result.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    let response = format!("success hashCal: {}", hex_str);
    ctx.ok(response.as_bytes());

    // for _ in 0..100000 {
    //     let mut hasher = Sha256::new();
    //     hasher.update(input);
    //     hasher.finalize();
    // }
    // ctx.ok("success hashCal".as_bytes());
}

// 大数计算：执行 10000 次模幂运算
#[no_mangle]
pub extern "C" fn bigNumCal() {
    let ctx = &mut sim_context::get_sim_context();

    let a = BigInt::from(2);
    let exp = BigInt::from(100_000u32);
    let modulus = BigInt::from(1_000_000_007u32);
    let mut result = BigInt::zero();

    for _ in 0..10000 {
        result = a.modpow(&exp, &modulus);
    }

    let response = format!("success bigNumCal: {}", result);
    ctx.ok(response.as_bytes());
}

#[no_mangle]
pub extern "C" fn normalStrconv() {
    let ctx = &mut sim_context::get_sim_context();

    let mut result1: i64 = 0;
    let mut result2: i64 = 0;
    let mut result3: i32 = 0;

    // 循环 10000 次数值转换运算
    for _ in 0..10000 {
        // strconv.ParseInt("1234567", 16, 32) 十六进制转i64，32位范围
        result1 = i64::from_str_radix("1234567", 16).unwrap_or(0);
        // strconv.ParseInt("1234567", 2, 64) 二进制转i64（非法二进制，返回0）
        result2 = i64::from_str_radix("1234567", 2).unwrap_or(0);
        // strconv.Atoi("1234567") 十进制转整数
        result3 = "1234567".parse::<i32>().unwrap_or(0);
    }

    let output = format!("success bigNumCal: {} {} {}", result1, result2, result3);
    ctx.ok(output.as_bytes());
}