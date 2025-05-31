/**
 * 类型转换工具模块
 * 负责处理对象类型的强制转换和验证
 */

/**
 * 根据类型定义强制转换对象类型
 * @param obj 需要转换的对象
 * @param typeDef 类型定义对象
 */
export function convertType(obj: any, typeDef: any): void {
  if (typeof obj !== typeof typeDef) {
    return
  }

  if (typeof obj === "object" && obj !== null) {
    for (const key in typeDef) {
      if (typeDef.hasOwnProperty(key)) {
        if (typeof obj[key] !== typeof typeDef[key]) {
          // 强制转换类型
           if (typeof typeDef[key] === "number") {
            if(obj[key] === null) {
              obj[key] = undefined
            }else {
              obj[key] = Number(obj[key])
            }
          } else if (typeof typeDef[key] === "string") {
            obj[key] = String(obj[key])
          } else if (typeof typeDef[key] === "boolean") {
            obj[key] = Boolean(obj[key])
          } else if (Array.isArray(typeDef[key])) {
            obj[key] = Array.isArray(obj[key]) ? obj[key] : []
          } else if (typeof typeDef[key] === "object") {
            obj[key] = typeof obj[key] === "object" ? obj[key] : {}
          }
        }
        convertType(obj[key], typeDef[key])
      }
    }
  }
}

/**
 * 检测字符串是否为有效的十六进制格式
 * @param str 待检测的字符串
 * @returns 是否为有效的十六进制格式
 */
export function isValidHexString(str: string): boolean {
  return /^[0-9A-Fa-f\s]+$/.test(str)
}

/**
 * 格式化十六进制字符串，移除空格
 * @param hexStr 十六进制字符串
 * @returns 格式化后的字符串
 */
export function formatHexString(hexStr: string): string {
  return hexStr.replace(/\s+/g, '')
}
