use contract_sdk_rust::sim_context;
use contract_sdk_rust::sim_context::SimContext;
use contract_sdk_rust::easycodec::*;
use sha2::{Sha256, Digest};
use num_bigint::{BigInt, ToBigInt};
use num_traits::Zero;
use hex;
const ERC721_INFO_MAP_NAME: &str = "erc721";
const BALANCE_INFO_MAP_NAME: &str = "balanceInfo";
const ACCOUNT_MAP_NAME: &str = "accountInfo";
const TOKEN_OWNER_MAP_NAME: &str = "tokenIds";
const TOKEN_INFO_MAP_NAME: &str = "tokenInfo";
const TOKEN_APPROVE_MAP_NAME: &str = "tokenApprove";
const OPERATOR_APPROVE_MAP_NAME: &str = "operatorApprove";
const TRUE_STRING: &str = "1";
const FALSE_STRING: &str = "0";
const ZERO_ADDR: &str = "0000000000000000000000000000000000000000";
const ZERO_ADDR_WITH_PREFIX: &str = "0x0000000000000000000000000000000000000000";
const MAX_SAFE_UINT256: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";  // 最大的 uint256 值
const MIN_SAFE_UINT256: &str = "0";  // 最小的 uint256 值

pub struct SafeUint256(BigInt);
struct TokenLatestTxInfo {
    tx_id: String,
    from: String,
    to: String,
    block_height: i32,
    ec: EasyCodec,
}
impl TokenLatestTxInfo {
    fn new_TokenLatestTxInfo(tx_id: String, from: String, to: String,block_height: i32) -> TokenLatestTxInfo {
        let mut ec = EasyCodec::new();
        ec.add_string("tx_id", tx_id.as_str());
        ec.add_string("from", from.as_str());
        ec.add_string("to", from.as_str());
        ec.add_i32("block_height", block_height);
        TokenLatestTxInfo {
            tx_id,
            block_height,
            from,
            to,
            ec,
        }
    }

    fn get_emit_event_data(&self) -> Vec<String> {
        let mut arr: Vec<String> = Vec::new();
        arr.push(self.tx_id.clone());
        arr.push(self.from.clone());
        arr.push(self.to.clone());
        arr.push(self.block_height.to_string());
        arr
    }

    fn to_json(&self) -> String {
        self.ec.to_json()
    }

    fn marshal(&self) -> Vec<u8> {
        self.ec.marshal()
    }

    fn unmarshal(data: &Vec<u8>) -> TokenLatestTxInfo {
        let ec = EasyCodec::new_with_bytes(data);
        TokenLatestTxInfo {
            tx_id: ec.get_string("tx_id").unwrap(),
            from: ec.get_string("from").unwrap(),
            to: ec.get_string("to").unwrap(),
            block_height: ec.get_i32("block_height").unwrap(),
            ec,
        }
    }
}


impl SafeUint256 {
    // Function to parse and check a SafeUint256 from a string
    pub fn parse_safe_uint256(x: &str) -> Option<SafeUint256> {
        let z = match BigInt::parse_bytes(x.as_bytes(), 10) {
            Some(val) => val,
            None => return None,  // if parsing fails, return None
        };

        // Check if the value is within the range of uint256
        let max = BigInt::parse_bytes(MAX_SAFE_UINT256.as_bytes(), 16).unwrap();  // parse max value
        let min = BigInt::parse_bytes(MIN_SAFE_UINT256.as_bytes(), 16).unwrap();  // parse min value

        if z > max || z < min {
            return None;  // out of bounds
        }

        Some(SafeUint256(z))  // valid SafeUint256
    }
    pub fn to_string(&self) -> String {
        self.0.to_str_radix(10) // 将 BigUint 转换为十进制字符串
    }
    pub fn safe_add(x: &SafeUint256, y: &SafeUint256) -> Option<SafeUint256> {
        let max_value = BigInt::parse_bytes(MAX_SAFE_UINT256.as_bytes(), 16).unwrap();
        let sum = &x.0 + &y.0;
        if sum > max_value {
            None
        } else {
            Some(SafeUint256(sum))
        }
    }

    pub fn safe_sub(x: &SafeUint256, y: &SafeUint256) -> Option<SafeUint256> {
        if x.0 < y.0 {
            Some(SafeUint256(BigInt::from(0)))
        } else {
            Some(SafeUint256(&x.0 - &y.0))
        }
    }

    pub fn safe_mul(x: &SafeUint256, y: &SafeUint256) -> Option<SafeUint256> {
        let max_value = BigInt::parse_bytes(MAX_SAFE_UINT256.as_bytes(), 16).unwrap();
        let product = &x.0 * &y.0;
        if product > max_value {
            None
        } else {
            Some(SafeUint256(product))
        }
    }

    pub fn safe_div(x: &SafeUint256, y: &SafeUint256) -> Option<SafeUint256> {
        if y.0.is_zero() {
            None
        } else {
            Some(SafeUint256(&x.0 / &y.0))
        }
    }
}


// 根据公钥计算地址
fn calc_address(pub_key: &str) -> String {
    return pub_key.to_string();
}
fn is_valid_address(addr: &str) -> bool {
    let addr_len = ZERO_ADDR.len();
    let addr_len_with_prefix = ZERO_ADDR_WITH_PREFIX.len();

    if addr.len() != addr_len && addr.len() != addr_len_with_prefix {
        return false;
    }

    let addr = addr.trim_start_matches("0x"); // 去掉前缀 0x
    return if hex::decode(addr).is_ok() {
        true
    } else {
        false
    }
}
fn is_zero_address(addr: &str) -> bool {
    return addr==ZERO_ADDR_WITH_PREFIX||addr==ZERO_ADDR;

}
fn update_erc721info(){
    let ctx = &mut sim_context::get_sim_context();
    let name=ctx.arg_as_utf8_str("name");
    let symbol=ctx.arg_as_utf8_str("symbol");
    let tokenURI=ctx.arg_as_utf8_str("tokenURI");
    if name.len()>0 {
        ctx.put_state(ERC721_INFO_MAP_NAME,"name",name.as_bytes());
    }
    if symbol.len()>0 {
        ctx.put_state(ERC721_INFO_MAP_NAME,"symbol",symbol.as_bytes());
    }
    if tokenURI.len()>0 {
        ctx.put_state(ERC721_INFO_MAP_NAME,"tokenURI",tokenURI.as_bytes());
    }
    let sender_key = ctx.get_sender_pub_key();
    let sender_address=calc_address(&sender_key);
    ctx.put_state(ERC721_INFO_MAP_NAME,"admin",sender_address.as_bytes());
}
//查看sender是否有被owner允许控制owner资产的权限
fn isApprovedForAll(owner: &str,sender: &str) ->bool{
    let ctx = &mut sim_context::get_sim_context();

    let approval_msg = format!("{}_give_approval_to_{}", owner, sender);
    let r = ctx.get_state(OPERATOR_APPROVE_MAP_NAME, &approval_msg);
    if r.is_err() {
        return false;
    }
    let approved = r.unwrap();
    if approved.len() == 0 {
        return false
    }
    return true;
}
//判断资产的所有者
// fn ownerOf(tokenId: &SafeUint256)->String{
//     let ctx = &mut sim_context::get_sim_context();
//     let token_id_str = tokenId.to_string();
//     let r = ctx.get_state(TOKEN_OWNER_MAP_NAME, &token_id_str);
//     if r.is_err() {
//         return "".to_string();
//     }
//     let owner = r.unwrap();
//     if owner.len() == 0 {
//         return "".to_string();
//     }
//     let owner_address =match String::from_utf8(owner) {
//         Ok(owner_str) => {
//             owner_str;
//         }
//         Err(e) => {
//             ctx.error("Failed to parse bytes as UTF-8");
//             return "".to_string();
//         }
//     };
//     return owner_address.to_string();
// }
//判断资产是否单独授给过别人
// fn getApproved(tokenId: &SafeUint256)->String{
//     let ctx = &mut sim_context::get_sim_context();
//     let token_id_str = tokenId.to_string();
//     let r = ctx.get_state(TOKEN_APPROVE_MAP_NAME, &token_id_str);
//     if r.is_err() {
//         return "".to_string();
//     }
//     let approveTo = r.unwrap();
//     if approveTo.len() == 0 {
//         return "".to_string();
//     }
//     let approveTo_address =match String::from_utf8(approveTo) {
//         Ok(approveTo_str) => {
//             approveTo_str;
//         }
//         Err(e) => {
//             ctx.error("Failed to parse bytes as UTF-8");
//             return "".to_string();
//         }
//     };
//     return approveTo_address.to_string();
//
// }
//查看账户（也就是地址）的余额
// fn getBalance(account: &str)->String{
//     let ctx = &mut sim_context::get_sim_context();
//     let r=ctx.get_state(BALANCE_INFO_MAP_NAME,account);
//     if r.is_err() {
//         ctx.error("get balance failed, err");
//         return "".to_string();
//     }
//     let balance = r.unwrap();
//     if balance.len() == 0 {
//         ctx.error("balance bytes invalid");
//         return "".to_string();
//     }
//     let balance_str =match String::from_utf8(balance) {
//         Ok(balance_str) => {
//             balance_str.to_string();
//         }
//         Err(e) => {
//             ctx.error("Failed to parse bytes as UTF-8");
//             return "".to_string();
//         }
//     };
//     return balance_str.to_string();
// }
fn getBalance(account: &str) -> String {
    let ctx = &mut sim_context::get_sim_context();

    // Fetch the balance for the account
    let r = ctx.get_state(BALANCE_INFO_MAP_NAME, account);

    // If there is an error fetching the balance, log and return an empty string
    if r.is_err() {
        ctx.error("get balance failed, err");
        return "".to_string();
    }

    // Unwrap the result and check if the balance is empty
    let mut balance = r.unwrap();
    if balance.len() == 0 {
        balance=b"0".to_vec();
    }

    // Attempt to convert the balance data into a string
    let balance_str = match String::from_utf8(balance) {
        Ok(balance_str) => balance_str,  // Correctly return the value
        Err(e) => {
            ctx.error("Failed to parse bytes as UTF-8");
            return "".to_string();
        }
    };

    // Return the balance as a string
    return balance_str;  // No need to call `.to_string()`, `balance_str` is already a `String`
}

fn ownerOf(tokenId: &SafeUint256) -> String {
    let ctx = &mut sim_context::get_sim_context();
    let token_id_str = tokenId.to_string();

    // Fetch the state of the token owner
    let r = ctx.get_state(TOKEN_OWNER_MAP_NAME, &token_id_str);

    // If there is an error fetching the state, return an empty string
    if r.is_err() {
        return "".to_string();
    }

    // Unwrap the result and check if the owner is empty
    let owner = r.unwrap();
    if owner.len() == 0 {
        return "".to_string();
    }

    // Attempt to convert the owner data into a string
    let owner_address = match String::from_utf8(owner) {
        Ok(owner_str) => owner_str,  // Correctly return the value
        Err(e) => {
            ctx.error("Failed to parse bytes as UTF-8");
            return "".to_string();
        }
    };

    // Return the owner address as a string
    return owner_address;  // No need to call `.to_string()`, `owner_address` is already a `String`
}
fn getApproved(tokenId: &SafeUint256) -> String {
    let ctx = &mut sim_context::get_sim_context();
    let token_id_str = tokenId.to_string();

    // Fetch the state of the token approval
    let r = ctx.get_state(TOKEN_APPROVE_MAP_NAME, &token_id_str);

    // If there is an error fetching the state, return an empty string
    if r.is_err() {
        return "".to_string();
    }

    // Unwrap the result and check if the approval is empty
    let approveTo = r.unwrap();
    if approveTo.len() == 0 {
        return "".to_string();
    }

    // Attempt to convert the approval data into a string
    let approveTo_address = match String::from_utf8(approveTo) {
        Ok(approveTo_str) => approveTo_str,  // Correctly return the value
        Err(e) => {
            ctx.error("Failed to parse bytes as UTF-8");
            return "".to_string();
        }
    };

    // Return the address as a string
    return approveTo_address;  // No need to call `.to_string()`, `approveTo_address` is already a `String`
}


//判断sender是否是tokenId这个资产的授权者或拥有者,或是这个资产单独授给过他
fn isApprovedOrOwner(sender: &str,tokenId: &SafeUint256)->bool{
    let ctx = &mut sim_context::get_sim_context();
    let owner_address=ownerOf(tokenId);
    if owner_address == sender{
        return true;
    }
    if isApprovedForAll(&owner_address,sender){
        return true;
    }
    let approveTo_address=getApproved(tokenId);
    if approveTo_address == sender{
        return true;
    }
    return false;
}
fn increaseTokenCountByOne(account: &str)->bool{
    let ctx = &mut sim_context::get_sim_context();
    let balance_str=getBalance(account);

    let originTokenCount =match SafeUint256::parse_safe_uint256(&balance_str) {
        Some(balance) => balance,
        None => {
            let err_msg = format!("Parse tokenId failed {}",balance_str.to_string() );
            ctx.error(&err_msg);
            return false;
        }
    };
    let one = SafeUint256::parse_safe_uint256("1").unwrap();
    let newTokenCount =match SafeUint256::safe_add(&originTokenCount, &one) {
        Some(result) => result,
        None => {
            ctx.error("balance count of from is overflow");
            return false;
        }
    };
    ctx.put_state(BALANCE_INFO_MAP_NAME,account,newTokenCount.to_string().as_bytes());
    return true;
}
fn decreaseTokenCountByOne(account: &str)->bool{
    let ctx = &mut sim_context::get_sim_context();
    let balance_str=getBalance(account);
    let originTokenCount =match SafeUint256::parse_safe_uint256(&balance_str) {
        Some(balance) => balance,
        None => {
            ctx.error("Parse tokenId failed");
            return false;
        }
    };
    let one = SafeUint256::parse_safe_uint256("1").unwrap();
    let newTokenCount =match SafeUint256::safe_sub(&originTokenCount, &one) {
        Some(result) => result,
        None => {
            ctx.error("token count of account is overflow");
            return false;
        }
    };
    ctx.put_state(BALANCE_INFO_MAP_NAME,account,newTokenCount.to_string().as_bytes());
    return true;
}
fn setTokenOwner(to: &str,tokenId: &SafeUint256)->bool{
    let ctx = &mut sim_context::get_sim_context();
    ctx.put_state(TOKEN_OWNER_MAP_NAME,&tokenId.to_string(),to.as_bytes());
    return true;
}
fn setAccountToken(from:&str,to: &str,tokenId: &SafeUint256)->bool{
    let ctx = &mut sim_context::get_sim_context();
    let mut account_msg = format!("{}_has_{}", to,tokenId.to_string() );
    ctx.put_state(ACCOUNT_MAP_NAME,&account_msg,TRUE_STRING.as_bytes());
    if is_zero_address(from){
        return true;
    }
    account_msg = format!("{}_has_{}", from,tokenId.to_string() );
    ctx.delete_state(ACCOUNT_MAP_NAME,&account_msg);
    return true;
}
fn setMetadata(tokenId: &SafeUint256,metadata: &str)->bool{
    let ctx = &mut sim_context::get_sim_context();
    if metadata.len() > 0{
        ctx.put_state(TOKEN_INFO_MAP_NAME,&tokenId.to_string(),metadata.as_bytes());
    }else{
        ctx.error("metadata cannot be empty");
        return false;
    }
    return true;
}
fn setTokenLatestTxInfo(tokenId: &SafeUint256,from:&str,to: &str)->bool{
    let ctx = &mut sim_context::get_sim_context();
    let txId=ctx.get_tx_id();
    let blockHeight=ctx.get_block_height();
    //contract-sdk-rust没有取时间戳的接口
    let tx_info = TokenLatestTxInfo::new_TokenLatestTxInfo(txId,from.to_string(),to.to_string(),blockHeight as i32);
    let tx_info_msg = format!("{}_latestTxInfo", tokenId.to_string() );

    ctx.put_state(TOKEN_INFO_MAP_NAME,&tx_info_msg,tx_info.marshal().as_slice());
    return true;
}
//转移资产
fn transfer(from: &str,to:&str,tokenId: &SafeUint256)->bool{
    let ctx = &mut sim_context::get_sim_context();
    let owner_address=ownerOf(tokenId);
    //如果from根本不是资产所有者就不转
    if owner_address != from{
        // ctx.error("ERC721: transfer from incorrect owner");
        // return false;
    }

    if !is_valid_address(to){
        ctx.error("ERC20: transfer to the invalid address");
        return false;
    }

    if is_zero_address(to){
        ctx.error("ERC20: transfer to the zero address");
        return false;
    }
    //删除原本token的有权者
    ctx.delete_state(TOKEN_APPROVE_MAP_NAME,&tokenId.to_string());

    if !decreaseTokenCountByOne(from){
       return false;
    }

    if !increaseTokenCountByOne(to){
        return false;
    }

    if !setTokenOwner(to,tokenId){
        return false;
    }

    if !setAccountToken(from,to,tokenId){
        return false;
    }

    if !setTokenLatestTxInfo(tokenId,from,to){
        return false;
    }
    return true;
}
fn minted(tokenId: &SafeUint256)->bool{
    let owner_address=ownerOf(tokenId);
    if owner_address.len()>0 && is_zero_address(&owner_address){
        return true;
    }
    return false;
}
// 合约安装函数
#[no_mangle]
pub extern "C" fn init_contract() {
    let ctx = &mut sim_context::get_sim_context();
    update_erc721info();
    ctx.ok("Init contract success".as_bytes());
}
//合约更新函数
#[no_mangle]
pub extern "C" fn upgrade() {
    let ctx = &mut sim_context::get_sim_context();
    update_erc721info();
    ctx.ok("Upgrade contract success".as_bytes());
}

//这个 setApprovalForAll2 函数是我自己改写的，授权调用者（msg.sender）对于第三方（operator）所有资产的管理权，approved为true表示有权，false表示无权
#[no_mangle]
pub extern "C" fn setApprovalForAll2() {

    let ctx = &mut sim_context::get_sim_context();
    let approvalFrom=ctx.arg_as_utf8_str("approvalFrom");
    if approvalFrom.is_empty() ||!is_valid_address(&approvalFrom) {
        ctx.error("the address of approvalFrom should not be empty");
    }
    let sender_key = ctx.get_sender_pub_key();
    let sender_address=calc_address(&sender_key);
    if sender_address==approvalFrom {
        ctx.ok("ERC721: approve to caller".as_bytes());
    }
    let approval_msg = format!("{}_give_approval_to_{}", approvalFrom,sender_address );

    ctx.put_state(OPERATOR_APPROVE_MAP_NAME,&approval_msg,"1".as_bytes());
    ctx.ok("setApprovalForAll2 success".as_bytes());
}
#[no_mangle]
pub extern "C" fn transferFrom() {
    let ctx = &mut sim_context::get_sim_context();
    let from=ctx.arg_as_utf8_str("from");
    let to=ctx.arg_as_utf8_str("to");
    let tokenId_str=ctx.arg_as_utf8_str("tokenId");
    let tokenId =match SafeUint256::parse_safe_uint256(&tokenId_str) {
        Some(token_id) => token_id,
        None => {
            ctx.error("Parse tokenId failed");
            return;
        }
    };
    let sender_key = ctx.get_sender_pub_key();
    let sender_address=calc_address(&sender_key);
    //如果要转移过去的对象不是资产的所有者或有权者，就不转
    if !isApprovedOrOwner(&sender_address,&tokenId){
        // ctx.error("ERC721: caller is not token owner or approved");
    }
    if transfer(&from,&to,&tokenId){
        ctx.ok("transfer success".as_bytes());
    }else{
        ctx.error("ERC721: transfer failed");
    }
}

#[no_mangle]
pub extern "C" fn mint() {
    let ctx = &mut sim_context::get_sim_context();
    let to=ctx.arg_as_utf8_str("to");
    let metadata=ctx.arg_as_utf8_str("metadata");
    let tokenId_str=ctx.arg_as_utf8_str("tokenId");
    let tokenId =match SafeUint256::parse_safe_uint256(&tokenId_str) {
        Some(token_id) => token_id,
        None => {
            ctx.error("Parse tokenId failed");
            return;
        }
    };
    if !is_valid_address(&to){
        ctx.error("ERC721: mint to invalid address");
        return;
    }
    if is_zero_address(&to){
        ctx.error("ERC721: mint to the zero address");
        return;
    }
    if minted(&tokenId){
        ctx.error("duplicated token");
        return;
    }
    //这里没写判断管理员权限的语句


    if !increaseTokenCountByOne(&to){
        ctx.error("increaseTokenCountByOne failed");
        return;
    }

    if !setTokenOwner(&to,&tokenId){
        ctx.error("setTokenOwner failed");
        return;
    }

    if !setAccountToken(ZERO_ADDR,&to,&tokenId){
        ctx.error("setAccountToken failed");
        return;
    }

    if !setMetadata(&tokenId,&metadata){
        ctx.error("setMetadata failed");
        return;
    }


    if !setTokenLatestTxInfo(&tokenId,ZERO_ADDR,&to){
        ctx.error("setTokenLatestTxInfo failed");
        return;
    }
    ctx.ok("mint success".as_bytes());
}