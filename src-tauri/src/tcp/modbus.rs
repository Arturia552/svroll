use anyhow::Result;
use bytes::{Bytes, BytesMut, BufMut, Buf};
use crc16::{State, MODBUS};
use tracing::info;

/// Modbus RTU帧结构
/// 
/// 表示完整的Modbus RTU通信帧，专用于RTU协议
/// 支持标准的读寄存器响应格式：从站地址 + 功能码 + 数据部分 + CRC16
#[derive(Debug, Clone)]
pub struct ModbusFrame {
    /// 从站地址 (1字节)
    pub unit_id: u8,
    /// 功能码 (1字节)，定义执行的操作类型
    pub function_code: u8,
    /// 数据部分 (可变长度)
    pub data: Vec<u8>,
    /// CRC16校验值 (2字节，小端序)
    pub crc: u16,
    /// 帧总长度 (字节数)
    pub len: usize,
}

impl ModbusFrame {
    /// 创建新的Modbus RTU帧
    /// 
    /// 初始化所有字段为默认值
    pub fn new() -> Self {
        ModbusFrame {
            unit_id: 0,
            function_code: 0,
            data: Vec::new(),
            crc: 0,
            len: 4, // 最小长度：从站地址(1) + 功能码(1) + CRC16(2)
        }
    }

    /// 创建读寄存器响应帧
    /// 
    /// # 参数
    /// * `unit_id` - 从站地址
    /// * `function_code` - 功能码 (0x03 或 0x04)
    /// * `register_data` - 寄存器数据，每个寄存器2字节，大端序
    pub fn new_read_response(unit_id: u8, function_code: u8, register_data: &[u16]) -> Self {
        let mut data = Vec::new();
        
        // 添加字节计数
        let byte_count = (register_data.len() * 2) as u8;
        data.push(byte_count);
        
        // 添加寄存器数据（大端序）
        for register in register_data {
            data.extend_from_slice(&register.to_be_bytes());
        }
        
        let len = 4 + data.len(); // 从站地址(1) + 功能码(1) + 数据部分 + CRC16(2)
        
        ModbusFrame {
            unit_id,
            function_code,
            data,
            crc: 0, // 将在encode时计算
            len,
        }
    }

    /// 尝试从流缓冲区解析Modbus RTU帧
    /// 
    /// 适用于tokio stream流式处理，能够处理不完整的帧数据
    /// 支持标准的Modbus RTU格式：从站地址 + 功能码 + 数据部分 + CRC16
    /// 
    /// # 参数
    /// * `buffer` - 流缓冲区，可能包含不完整或多个帧的数据
    /// 
    /// # 返回
    /// - Ok(true): 成功解析完整帧，已从buffer中移除
    /// - Ok(false): 数据不足，需要等待更多数据
    /// - Err: 解析错误或CRC校验失败
    pub fn parse_frame(&mut self, buffer: &mut BytesMut) -> Result<bool> {
        // 检查最小长度要求：1字节从站地址 + 1字节功能码 + 2字节CRC
        if buffer.len() < 4 {
            return Ok(false); // 数据不足，等待更多数据
        }

        // 先读取帧头来确定数据长度，但不消费buffer
        let unit_id = buffer[0];
        let function_code = buffer[1];
        
        // 根据功能码确定期望的帧长度
        let expected_frame_len = match function_code {
            0x01 | 0x02 => {
                // 读线圈/离散输入响应：单元ID(1) + 功能码(1) + 字节计数(1) + 数据 + CRC(2)
                if buffer.len() < 3 {
                    return Ok(false); // 需要至少3字节才能读取字节计数
                }
                let byte_count = buffer[2] as usize;
                5 + byte_count // 单元ID + 功能码 + 字节计数 + 数据 + CRC
            },
            0x03 | 0x04 => {
                // 读寄存器响应：单元ID(1) + 功能码(1) + 字节计数(1) + 数据 + CRC(2)
                if buffer.len() < 3 {
                    return Ok(false); // 需要至少3字节才能读取字节计数
                }
                let byte_count = buffer[2] as usize;
                5 + byte_count // 单元ID + 功能码 + 字节计数 + 数据 + CRC
            },
            _ => {
                8
            }
        };

        // 检查是否有足够的数据来构成完整帧
        if buffer.len() < expected_frame_len {
            return Ok(false); // 数据不足，等待更多数据
        }
        // 现在我们有足够的数据，开始解析
        let frame_data = &buffer[..expected_frame_len];
        
        // 计算并验证CRC
        let crc_data = &frame_data[..expected_frame_len - 2];
        let calculated_crc = State::<MODBUS>::calculate(crc_data);
        
        let received_crc = u16::from_le_bytes([
            frame_data[expected_frame_len - 1], 
            frame_data[expected_frame_len - 2],
        ]);

        if received_crc != calculated_crc {
            return Err(anyhow::anyhow!(
                "RTU帧CRC校验失败: 收到 0x{:04X}, 计算得到 0x{:04X}",
                received_crc,
                calculated_crc
            ));
        }

        // CRC校验通过，解析帧数据
        self.unit_id = unit_id;
        self.function_code = function_code;
        
        // 提取数据部分（跳过单元ID、功能码，排除CRC）
        let data_start = 2;
        let data_end = expected_frame_len - 2;
        if data_end > data_start {
            self.data = frame_data[data_start..data_end].to_vec();
        } else {
            self.data.clear();
        }
        
        self.crc = received_crc;
        self.len = expected_frame_len;

        // 从buffer中移除已解析的帧数据
        let _ = buffer.split_to(expected_frame_len);
        
        Ok(true) // 成功解析完整帧
    }

    /// 解析读寄存器响应数据
    /// 
    /// 从响应帧的数据部分提取寄存器值
    /// 适用于功能码 0x03(读保持寄存器) 和 0x04(读输入寄存器) 的响应
    /// 
    /// # 返回
    /// 成功返回寄存器值数组，失败返回错误
    pub fn parse_read_response(&self) -> Result<Vec<u16>> {
        // 检查功能码
        if self.function_code != 0x03 && self.function_code != 0x04 {
            return Err(anyhow::anyhow!(
                "不支持的功能码，期望 0x03 或 0x04，实际: 0x{:02X}",
                self.function_code
            ));
        }

        // 检查数据长度
        if self.data.is_empty() {
            return Err(anyhow::anyhow!("读寄存器响应数据为空"));
        }

        // 第一个字节是字节计数
        let byte_count = self.data[0] as usize;
        
        // 验证字节计数
        if byte_count + 1 != self.data.len() {
            return Err(anyhow::anyhow!(
                "字节计数不匹配：期望 {}, 实际数据长度 {}",
                byte_count + 1,
                self.data.len()
            ));
        }

        // 验证字节计数是否为偶数（每个寄存器2字节）
        if byte_count % 2 != 0 {
            return Err(anyhow::anyhow!(
                "字节计数必须为偶数，实际: {}",
                byte_count
            ));
        }

        // 解析寄存器数据（大端序）
        let mut registers = Vec::new();
        let register_data = &self.data[1..]; // 跳过字节计数字段
        
        for chunk in register_data.chunks_exact(2) {
            let register_value = u16::from_be_bytes([chunk[0], chunk[1]]);
            registers.push(register_value);
        }

        Ok(registers)
    }



    /// 获取数据部分的字节计数（仅适用于读寄存器响应）
    /// 
    /// # 返回
    /// 成功返回字节计数，失败返回错误
    pub fn get_byte_count(&self) -> Result<u8> {
        if self.data.is_empty() {
            return Err(anyhow::anyhow!("数据部分为空"));
        }
        Ok(self.data[0])
    }

    /// 获取帧的总长度
    /// 
    /// # 返回
    /// 帧的总字节数
    pub fn total_length(&self) -> usize {
        self.len
    }

    /// 更新帧的总长度
    /// 
    /// 根据当前的数据内容重新计算并更新长度字段
    pub fn update_length(&mut self) {
        self.len = 4 + self.data.len(); // 从站地址(1) + 功能码(1) + 数据部分 + CRC16(2)
    }

    /// 编码Modbus RTU帧数据
    /// 
    /// 将结构化的Modbus RTU帧编码为字节序列
    /// 生成标准的Modbus RTU格式：从站地址 + 功能码 + 数据部分 + CRC16
    /// 
    /// # 返回
    /// 包含完整Modbus RTU帧的字节数据
    pub fn encode_frame(&self) -> Bytes {
        let mut buffer = BytesMut::new();

        // 添加从站地址 (1字节)
        buffer.put_u8(self.unit_id);
        
        // 添加功能码 (1字节)
        buffer.put_u8(self.function_code);

        // 添加数据部分
        buffer.put_slice(&self.data);

        // 计算CRC校验值
        let calculated_crc = State::<MODBUS>::calculate(&buffer);

        // 添加CRC校验值 (小端序，2字节)
        buffer.put_u16_le(calculated_crc);

        buffer.freeze()
    }
}


impl TryFrom<&mut BytesMut> for ModbusFrame {
    type Error = anyhow::Error;
    fn try_from(buffer: &mut BytesMut) -> Result<Self, Self::Error> {
        let mut frame = ModbusFrame::new();
        match frame.parse_frame(buffer)? {
            true => Ok(frame),
            false => Err(anyhow::anyhow!("数据不足，无法解析完整帧")),
        }
    }
}

impl From<ModbusFrame> for Bytes {
    fn from(frame: ModbusFrame) -> Self {
        frame.encode_frame()
    }
}
