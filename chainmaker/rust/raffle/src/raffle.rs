use contract_sdk_rust::sim_context;
use contract_sdk_rust::sim_context::SimContext;
use contract_sdk_rust::easycodec::*;

const PEOPLES_KEY: &str = "peoples";
const QUERY_ERROR_MSG: &str = "get peoples data failed";

// 抽奖参与者信息
struct People {
    num: i32,
    name: String,
    ec: EasyCodec,
}

impl People {
    fn new_people(num: i32, name: String) -> People {
        let mut ec = EasyCodec::new();
        ec.add_i32("num", num);
        ec.add_string("name", name.as_str());
        People { num, name, ec }
    }

    fn to_json(&self) -> String {
        self.ec.to_json()
    }

    fn marshal(&self) -> Vec<u8> {
        self.ec.marshal()
    }

    fn unmarshal(data: &Vec<u8>) -> People {
        let ec = EasyCodec::new_with_bytes(data);
        People {
            num: ec.get_i32("num").unwrap(),
            name: ec.get_string("name").unwrap(),
            ec,
        }
    }
}

// 参与者集合
struct Peoples {
    peoples: Vec<People>,
    ec: EasyCodec,
}

impl Peoples {
    fn new_peoples(peoples: Vec<People>) -> Peoples {
        let mut ec = EasyCodec::new();
        // 将 peoples 序列化存储到 ec 中
        // 这里使用一个特殊的 key 来存储整个列表
        for (i, people) in peoples.iter().enumerate() {
            let key = format!("people_{}", i);
            ec.add_bytes(&key, people.marshal());
        }
        ec.add_i32("count", peoples.len() as i32);
        Peoples { peoples, ec }
    }

    fn to_json(&self) -> String {
        // 构建 JSON 字符串，格式: {"peoples":[{"num":1,"name":"xx"},...]}
        let mut json = String::from("{\"peoples\":[");
        for (i, people) in self.peoples.iter().enumerate() {
            if i > 0 {
                json.push_str(",");
            }
            json.push_str(&people.to_json());
        }
        json.push_str("]}");
        json
    }

    fn marshal(&self) -> Vec<u8> {
        self.ec.marshal()
    }

    fn unmarshal(data: &Vec<u8>) -> Peoples {
        let ec = EasyCodec::new_with_bytes(data);
        let count = ec.get_i32("count").unwrap_or(0);
        let mut peoples = Vec::new();
        for i in 0..count {
            let key = format!("people_{}", i);
            let people_data = ec.get_bytes(&key).unwrap_or(Vec::new());
            if people_data.len() > 0 {
                let people = People::unmarshal(&people_data);
                peoples.push(people);
            }
        }
        Peoples { peoples, ec }
    }

    fn len(&self) -> usize {
        self.peoples.len()
    }

    fn get(&self, index: usize) -> Option<&People> {
        self.peoples.get(index)
    }

    // 移除指定位置的元素
    fn remove(&mut self, index: usize) -> Option<People> {
        if index < self.peoples.len() {
            Some(self.peoples.remove(index))
        } else {
            None
        }
    }

    // 更新 ec 数据
    fn update_ec(&mut self) {
        let mut ec = EasyCodec::new();
        for (i, people) in self.peoples.iter().enumerate() {
            let key = format!("people_{}", i);
            ec.add_bytes(&key, people.marshal());
        }
        ec.add_i32("count", self.peoples.len() as i32);
        self.ec = ec;
    }
}

// 合约安装函数
#[no_mangle]
pub extern "C" fn init_contract() {
    let ctx = &mut sim_context::get_sim_context();
    ctx.ok("Init contract success".as_bytes());
}

// 合约更新函数
#[no_mangle]
pub extern "C" fn upgrade() {
    let ctx = &mut sim_context::get_sim_context();
    ctx.ok("Upgrade contract success".as_bytes());
}

// 批量注册参与者
#[no_mangle]
pub extern "C" fn register_all() {
    let ctx = &mut sim_context::get_sim_context();

    // 获取参数
    let peoples_str = ctx.arg_as_utf8_str("peoples");

    // 发送事件
    let mut event_data: Vec<String> = Vec::new();
    event_data.push(peoples_str.clone());
    ctx.emit_event("peoples", &event_data);

    if peoples_str.is_empty() {
        ctx.error("peoples param should not be empty");
        return;
    }

    // 解析数据 - 手动解析 JSON 格式的字符串
    // 期望格式: {"peoples":[{"num":1,"name":"xxx"},{"num":2,"name":"yyy"}]}
    let mut peoples = Vec::new();
    // 简单解析：查找 "peoples":[ 和 ] 之间的内容
    let start = match peoples_str.find("\"peoples\":[") {
        Some(s) => s + 11, // 跳过 "peoples":[
        None => {
            ctx.error("invalid peoples format: missing peoples array");
            return;
        }
    };
    let end = match peoples_str.rfind("]}") {
        Some(e) => e + 1,
        None => {
            ctx.error("invalid peoples format: missing closing braces");
            return;
        }
    };

    if start >= end {
        ctx.error("invalid peoples format: empty array");
        return;
    }

    let array_content = &peoples_str[start..end];
    // 解析每个 people 对象: {"num":1,"name":"xxx"}
    let mut pos = 0;
    let bytes = array_content.as_bytes();
    while pos < bytes.len() {
        // 查找 '{'
        let obj_start = match array_content[pos..].find('{') {
            Some(s) => s + pos,
            None => break,
        };
        // 查找对应的 '}'
        let mut brace_count = 0;
        let mut obj_end = obj_start;
        for i in obj_start..array_content.len() {
            let c = array_content.chars().nth(i).unwrap();
            if c == '{' {
                brace_count += 1;
            } else if c == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    obj_end = i;
                    break;
                }
            }
        }

        if obj_end > obj_start {
            let obj_str = &array_content[obj_start..=obj_end];
            // 解析 num
            let num_start = match obj_str.find("\"num\":") {
                Some(s) => s + 6,
                None => {
                    ctx.error("invalid people format: missing num");
                    return;
                }
            };
            let num_end = match obj_str[num_start..].find(',') {
                Some(e) => num_start + e,
                None => {
                    // 可能是最后一个字段
                    match obj_str[num_start..].find('}') {
                        Some(e) => num_start + e,
                        None => {
                            ctx.error("invalid people format: cannot find num end");
                            return;
                        }
                    }
                }
            };
            let num_str = &obj_str[num_start..num_end].trim();
            let num = match num_str.parse::<i32>() {
                Ok(n) => n,
                Err(e) => {
                    ctx.error("invalid people format: num is not integer");
                    return;
                }
            };

            // 解析 name
            let name_start = match obj_str.find("\"name\":\"") {
                Some(s) => s + 8,
                None => {
                    ctx.error("invalid people format: missing name");
                    return;
                }
            };
            let name_end = match obj_str[name_start..].find('\"') {
                Some(e) => name_start + e,
                None => {
                    ctx.error("invalid people format: cannot find name end");
                    return;
                }
            };
            let name = obj_str[name_start..name_end].to_string();

            // 校验数据
            if name.is_empty() {
                ctx.error("name should not be empty");
                return;
            }

            let people = People::new_people(num, name);
            peoples.push(people);
            pos = obj_end + 1;
        } else {
            break;
        }
    }

    if peoples.is_empty() {
        ctx.error("no valid peoples found");
        return;
    }

    let peoples_obj = Peoples::new_peoples(peoples);

    // 存储数据
    let code = ctx.put_state(PEOPLES_KEY, "", peoples_obj.marshal().as_slice());
    if code != 0 {
        ctx.error("save peoples failed");
        return;
    }

    ctx.ok("ok".as_bytes());
}

// 抽奖函数
#[no_mangle]
pub extern "C" fn raffle() {
    let ctx = &mut sim_context::get_sim_context();

    // 获取参数
    let level = ctx.arg_as_utf8_str("level");
    let arg_timestamp = ctx.arg_as_utf8_str("timestamp");

    if level.is_empty() {
        ctx.error("level should not be empty!");
        return;
    }
    if arg_timestamp.is_empty() {
        ctx.error("argTimestamp should not be empty!");
        return;
    }

    // 获取参与者数据
    let peoples_data = match ctx.get_state(PEOPLES_KEY, "") {
        Ok(data) => data,
        Err(e) => {
            ctx.error(QUERY_ERROR_MSG);
            return;
        }
    };

    if peoples_data.is_empty() {
        ctx.error("no peoples data found");
        return;
    }

    let mut peoples_obj = Peoples::unmarshal(&peoples_data);

    if peoples_obj.len() == 0 {
        ctx.error("peoples list is empty");
        return;
    }

    // 计算中奖位置
    let hash_val = bkdr_hash(&arg_timestamp);
    let num = (hash_val % peoples_obj.len() as i32) as usize;
    ctx.log(&format!("raffle position: {}", num));

    // 获取中奖者
    let result_people = match peoples_obj.get(num) {
        Some(p) => p,
        None => {
            ctx.error("invalid raffle position");
            return;
        }
    };
    let result = format!("num: {}, name: {}, level: {}", result_people.num, result_people.name, level);

    // 更新参与者列表（移除中奖者）
    peoples_obj.remove(num);
    peoples_obj.update_ec();

    // 保存新数据
    let code = ctx.put_state(PEOPLES_KEY, "", peoples_obj.marshal().as_slice());
    if code != 0 {
        ctx.error("put new peoples data failed");
        return;
    }

    ctx.ok(result.as_bytes());
}

// 注册并抽奖（组合函数）
#[no_mangle]
pub extern "C" fn registRaffle() {
    let ctx = &mut sim_context::get_sim_context();

    // 获取参数
    let peoples_str = ctx.arg_as_utf8_str("peoples");

    // 发送事件
    let mut event_data: Vec<String> = Vec::new();
    event_data.push(peoples_str.clone());
    ctx.emit_event("peoples", &event_data);

    if peoples_str.is_empty() {
        ctx.error("peoples param should not be empty");
        return;
    }

    // 解析数据 - 手动解析 JSON
    let mut peoples = Vec::new();
    let start = match peoples_str.find("\"peoples\":[") {
        Some(s) => s + 11,
        None => {
            ctx.error("invalid peoples format: missing peoples array");
            return;
        }
    };
    let end = match peoples_str.rfind("]}") {
        Some(e) => e + 1,
        None => {
            ctx.error("invalid peoples format: missing closing braces");
            return;
        }
    };

    if start >= end {
        ctx.error("invalid peoples format: empty array");
        return;
    }

    let array_content = &peoples_str[start..end];
    let mut pos = 0;
    while pos < array_content.len() {
        let obj_start = match array_content[pos..].find('{') {
            Some(s) => s + pos,
            None => break,
        };
        let mut brace_count = 0;
        let mut obj_end = obj_start;
        for i in obj_start..array_content.len() {
            let c = array_content.chars().nth(i).unwrap();
            if c == '{' {
                brace_count += 1;
            } else if c == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    obj_end = i;
                    break;
                }
            }
        }

        if obj_end > obj_start {
            let obj_str = &array_content[obj_start..=obj_end];
            let num_start = match obj_str.find("\"num\":") {
                Some(s) => s + 6,
                None => {
                    ctx.error("invalid people format: missing num");
                    return;
                }
            };
            let num_end = match obj_str[num_start..].find(',') {
                Some(e) => num_start + e,
                None => {
                    match obj_str[num_start..].find('}') {
                        Some(e) => num_start + e,
                        None => {
                            ctx.error("invalid people format: cannot find num end");
                            return;
                        }
                    }
                }
            };
            let num_str = &obj_str[num_start..num_end].trim();
            let num = match num_str.parse::<i32>() {
                Ok(n) => n,
                Err(e) => {
                    ctx.error("invalid people format: num is not integer");
                    return;
                }
            };

            let name_start = match obj_str.find("\"name\":\"") {
                Some(s) => s + 8,
                None => {
                    ctx.error("invalid people format: missing name");
                    return;
                }
            };
            let name_end = match obj_str[name_start..].find('\"') {
                Some(e) => name_start + e,
                None => {
                    ctx.error("invalid people format: cannot find name end");
                    return;
                }
            };
            let name = obj_str[name_start..name_end].to_string();

            if name.is_empty() {
                ctx.error("name should not be empty");
                return;
            }

            let people = People::new_people(num, name);
            peoples.push(people);
            pos = obj_end + 1;
        } else {
            break;
        }
    }

    if peoples.is_empty() {
        ctx.error("no valid peoples found");
        return;
    }

    let mut peoples_obj = Peoples::new_peoples(peoples);

    // 存储数据
    let code = ctx.put_state(PEOPLES_KEY, "", peoples_obj.marshal().as_slice());
    if code != 0 {
        ctx.error("save peoples failed");
        return;
    }

    // 获取抽奖参数
    let level = ctx.arg_as_utf8_str("level");
    let arg_timestamp = ctx.arg_as_utf8_str("timestamp");

    if level.is_empty() {
        ctx.error("level should not be empty!");
        return;
    }
    if arg_timestamp.is_empty() {
        ctx.error("argTimestamp should not be empty!");
        return;
    }

    // 重新获取数据（确保是最新的）
    let peoples_data = match ctx.get_state(PEOPLES_KEY, "") {
        Ok(data) => data,
        Err(e) => {
            ctx.error(QUERY_ERROR_MSG);
            return;
        }
    };

    peoples_obj = Peoples::unmarshal(&peoples_data);

    if peoples_obj.len() == 0 {
        ctx.error("peoples list is empty");
        return;
    }

    // 计算中奖位置
    let hash_val = bkdr_hash(&arg_timestamp);
    let num = (hash_val % peoples_obj.len() as i32) as usize;
    ctx.log(&format!("raffle position: {}", num));

    // 获取中奖者
    let result_people = match peoples_obj.get(num) {
        Some(p) => p,
        None => {
            ctx.error("invalid raffle position");
            return;
        }
    };
    let result = format!("num: {}, name: {}, level: {}", result_people.num, result_people.name, level);

    // 更新参与者列表（移除中奖者）
    peoples_obj.remove(num);
    peoples_obj.update_ec();

    // 保存新数据
    let code = ctx.put_state(PEOPLES_KEY, "", peoples_obj.marshal().as_slice());
    if code != 0 {
        ctx.error("put new peoples data failed");
        return;
    }

    ctx.ok(result.as_bytes());
}

// 查询参与者
#[no_mangle]
pub extern "C" fn query() {
    let ctx = &mut sim_context::get_sim_context();

    let data = match ctx.get_state(PEOPLES_KEY, "") {
        Ok(d) => d,
        Err(e) => {
            ctx.error(QUERY_ERROR_MSG);
            return;
        }
    };

    if data.is_empty() {
        ctx.ok("{\"peoples\":[]}".as_bytes());
        return;
    }

    let peoples_obj = Peoples::unmarshal(&data);
    let json_str = peoples_obj.to_json();
    ctx.ok(json_str.as_bytes());
}

// BKDR哈希算法
fn bkdr_hash(input: &str) -> i32 {
    let mut hash: i32 = 0;
    let seed: i32 = 131;
    for c in input.chars() {
        hash = hash.wrapping_mul(seed).wrapping_add(c as i32);
    }
    hash & 0x7FFFFFFF // 保证正数
}