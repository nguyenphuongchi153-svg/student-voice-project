
#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Map, symbol_short, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Pending = 0,
    Escalated = 1,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Feedback {
    pub id: u64,
    pub content: String,
    pub student_hash: String,
    pub votes: u32,
    pub status: Status,
}

const ADMIN: Symbol = symbol_short!("ADMIN");
const FCOUNT: Symbol = symbol_short!("FCOUNT");
const FEEDBACKS: Symbol = symbol_short!("FEEDBACKS");
const STUDENTS: Symbol = symbol_short!("STUDENTS");
const VOTERS: Symbol = symbol_short!("VOTERS");
const THRESHOLD: u32 = 3; // Chỉ cần 3 vote để test cho nhanh

#[contract]
pub struct StudentVoiceContract;

#[contractimpl]
impl StudentVoiceContract {
    // Đặt tên hàm viết liền để tránh lỗi giao diện
    pub fn setup(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN) {
            panic!("He thong da thiet lap!");
        }
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&FCOUNT, &0u64);
    }

    pub fn addstudent(env: Env, admin: Address, studenthash: String) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&ADMIN).expect("Hay setup truoc");
        if admin != stored_admin {
            panic!("Khong phai Admin!");
        }
        let mut students: Map<String, bool> = env.storage().instance().get(&STUDENTS).unwrap_or(Map::new(&env));
        students.set(studenthash, true);
        env.storage().instance().set(&STUDENTS, &students);
    }

    pub fn submit(env: Env, studenthash: String, content: String) -> u64 {
        let students: Map<String, bool> = env.storage().instance().get(&STUDENTS).expect("Chua co SV");
        // SỬA LỖI: Chuyển contains thành contains_key
        if !students.contains_key(studenthash.clone()) {
            panic!("SV chua xac thuc!");
        }
        let mut count: u64 = env.storage().instance().get(&FCOUNT).unwrap_or(0);
        let feedback = Feedback {
            id: count,
            content,
            student_hash: studenthash,
            votes: 0,
            status: Status::Pending,
        };
        let mut feedbacks: Map<u64, Feedback> = env.storage().instance().get(&FEEDBACKS).unwrap_or(Map::new(&env));
        feedbacks.set(count, feedback);
        env.storage().instance().set(&FEEDBACKS, &feedbacks);
        env.storage().instance().set(&FCOUNT, &(count + 1));
        count
    }

    pub fn vote(env: Env, studenthash: String, id: u64) {
        let students: Map<String, bool> = env.storage().instance().get(&STUDENTS).expect("Chua co SV");
        if !students.contains_key(studenthash.clone()) {
            panic!("Ban khong phai SV!");
        }
        let voter_key = (studenthash, id);
        let mut voters: Map<(String, u64), bool> = env.storage().instance().get(&VOTERS).unwrap_or(Map::new(&env));
        if voters.contains_key(voter_key.clone()) {
            panic!("Da vote!");
        }
        let mut feedbacks: Map<u64, Feedback> = env.storage().instance().get(&FEEDBACKS).unwrap();
        let mut f = feedbacks.get(id).expect("Loi ID");
        f.votes += 1;
        if f.votes >= THRESHOLD { f.status = Status::Escalated; }
        voters.set(voter_key, true);
        feedbacks.set(id, f);
        env.storage().instance().set(&FEEDBACKS, &feedbacks);
        env.storage().instance().set(&VOTERS, &voters);
    }

    pub fn view(env: Env, id: u64) -> Feedback {
        let feedbacks: Map<u64, Feedback> = env.storage().instance().get(&FEEDBACKS).expect("Chua co du lieu");
        feedbacks.get(id).expect("Khong tim thay")
    }
}