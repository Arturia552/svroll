use anyhow::Result;

#[derive(Debug, Clone)]
pub struct DataField {
    pub length: u8,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ModbusFrame {
    pub transaction_id: u16,
    pub protocol_id: u16,
    pub length: u16,
    pub unit_id: u8,
    pub function_code: u8,
    pub data_fields: Vec<DataField>,
    pub crc: u16,
}

impl ModbusFrame {
    pub fn new() -> Self {
        ModbusFrame {
            transaction_id: 0,
            protocol_id: 0,
            length: 0,
            unit_id: 0,
            function_code: 0,
            data_fields: Vec::new(),
            crc: 0,
        }
    }

    // CRC16 计算
    fn calculate_crc(data: &[u8]) -> u16 {
        let mut crc = 0xFFFF;
        for byte in data {
            crc ^= *byte as u16;
            for _ in 0..8 {
                if (crc & 0x0001) != 0 {
                    crc = (crc >> 1) ^ 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }
        crc
    }

    pub fn parse_frame(&mut self, buffer: &[u8]) -> Result<()> {
        // 检查最小长度要求：7字节报文头 + 1字节功能码 + 2字节CRC
        if buffer.len() < 9 {
            return Err(anyhow::anyhow!("数据长度不足"));
        }

        // 解析 Modbus TCP 报文头
        // 事务标识符 (2字节): 用于请求/响应匹配
        self.transaction_id = u16::from_be_bytes([buffer[0], buffer[1]]);
        // 协议标识符 (2字节): Modbus协议固定为0
        self.protocol_id = u16::from_be_bytes([buffer[2], buffer[3]]);
        // 长度字段 (2字节): 表示后续字节数
        self.length = u16::from_be_bytes([buffer[4], buffer[5]]);
        // 单元标识符 (1字节): 标识从站地址
        self.unit_id = buffer[6];
        // 功能码 (1字节): 定义要执行的操作
        self.function_code = buffer[7];

        // 解析数据字段部分
        let mut pos = 8; // 从第9个字节开始解析数据字段
        while pos < buffer.len() - 2 {
            // 减2是为了预留CRC校验的空间
            // 确保至少有一个字节用于长度字段
            if pos + 1 > buffer.len() - 2 {
                return Err(anyhow::anyhow!("数据字段长度错误"));
            }

            // 读取数据字段长度 (1字节)
            let field_length = buffer[pos];
            pos += 1;

            // 检查数据字段值的长度是否合法
            if pos + field_length as usize > buffer.len() - 2 {
                return Err(anyhow::anyhow!("数据字段值长度错误"));
            }

            // 提取数据字段值
            let field_value = buffer[pos..pos + field_length as usize].to_vec();
            pos += field_length as usize;

            // 将解析出的数据字段添加到列表中
            self.data_fields.push(DataField {
                length: field_length,
                value: field_value,
            });
        }

        // CRC校验处理
        // 获取接收到的CRC值 (小端序)
        let received_crc = u16::from_le_bytes([
            buffer[buffer.len() - 2], // CRC低字节
            buffer[buffer.len() - 1], // CRC高字节
        ]);

        // 计算除CRC外的所有数据的CRC值
        let calculated_crc = Self::calculate_crc(&buffer[..buffer.len() - 2]);

        // 比较接收到的CRC和计算得到的CRC
        if received_crc != calculated_crc {
            return Err(anyhow::anyhow!(
                "CRC校验失败: 收到 0x{:04X}, 计算得到 0x{:04X}",
                received_crc,
                calculated_crc
            ));
        }

        // 保存校验通过的CRC值
        self.crc = received_crc;
        Ok(())
    }
}
