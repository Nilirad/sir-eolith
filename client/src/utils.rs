use crate::connection::{ClientMsg, ServerMsg};

pub fn decrypt_message(message: ServerMsg) -> ClientMsg {
    /// Index of the first message byte containing data.
    const MESSAGE_PAYLOAD_START: usize = 3;

    let mut javascript_code = Vec::<u8>::new();
    let mut d = 0u8;
    let mut e = 23;
    
    for (i, byte) in message.into_iter().skip(MESSAGE_PAYLOAD_START).enumerate() {
        let mut b = byte as i32;
        if b <= 96 {
            b += 32;
        }
        b = (b - 97 - e) % 26;
        if b < 0 {
            b += 26;
        }
        d *= 16;
        d += b as u8;
        e += 17;

        if i % 2 == 1 {
            javascript_code.push(d);
            d = 0;
        }
    }

    let idba = (&javascript_code[7..=30]).to_vec();
    let mut b = 0;
    idba.iter().enumerate().map(
        |(i, byte)| {
            let mut d = 65;
            let mut a = *byte as i32;
            if a >= 97 {
                d += 32;
                a -= 32;
            }
            a -= 65;
            if i == 0 {
                b = 2 + a;
            }
            e = a + b;
            e = e % 26;
            b += 3 + a;
            (e + d) as u8
        }
    )
    .collect()
}