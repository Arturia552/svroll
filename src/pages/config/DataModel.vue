<template>
  <div v-loading="loading" class="data-model">
    <div class="header">
      <div class="title">
        数据模型配置
      </div>
      <div class="actions">
        <el-tooltip content="刷新数据结构" placement="top">
          <el-button type="primary" circle @click="refreshStructure">
            <el-icon>
              <refresh />
            </el-icon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="添加字段" placement="top">
          <el-button type="success" circle @click="showAddFieldDialog">
            <el-icon>
              <plus />
            </el-icon>
          </el-button>
        </el-tooltip>
        <el-tooltip content="导入模板" placement="top">
          <el-button type="info" circle @click="importTemplate">
            <el-icon>
              <download />
            </el-icon>
          </el-button>
        </el-tooltip>
      </div>
    </div>

    <el-divider />

    <div class="table-container">
      <el-table :data="config.fieldStruct"
                style="width: 100%"
                :height="ModelTableHeight"
                border
                stripe
                highlight-current-row>
        <el-table-column prop="fieldName" label="键值" min-width="20%" />

        <el-table-column prop="fieldType" label="数据类型" min-width="20%">
          <template #default="scope">
            <div class="cell-content">
              {{ getFieldTypeName(scope.row.fieldType) }}
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="minValue" label="最小值" min-width="15%">
          <template #default="scope">
            <div class="cell-content">
              {{ scope.row.minValue ?? "--" }}
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="maxValue" label="最大值" min-width="15%">
          <template #default="scope">
            <div class="cell-content">
              {{ scope.row.maxValue ?? "--" }}
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="possibleValues" label="有效值" min-width="20%">
          <template #default="scope">
            <div class="cell-content">
              <el-tag v-for="(item, index) in scope.row.possibleValues"
                      :key="index" 
                      class="possible-value-tag">
                <span> {{ item.value }}</span><span v-if="scope.row.fieldType === FieldTypeEnum.Enum"> ({{ item.probability }}%)</span>
              </el-tag>
            </div>
          </template>
        </el-table-column>

        <el-table-column align="center" label="操作" width="100">
          <template #default="scope">
            <el-button type="primary" size="small" @click="showEditFieldDialog(scope.row)">
              编辑
            </el-button>
          </template>
        </el-table-column>

        <template #empty>
          <div class="empty-data">
            <el-empty description="暂无数据结构" :image-size="120">
              <template #extra>
                <el-button type="primary" @click="showAddFieldDialog">
                  添加字段
                </el-button>
              </template>
            </el-empty>
          </div>
        </template>
      </el-table>
    </div>

    <el-dialog v-model="addFieldDialogVisible"
               title="添加字段"
               width="500px"
               :close-on-click-modal="false"
               :append-to-body="true">
      <el-form ref="fieldFormRef"
               :model="newField"
               label-width="100px"
               :rules="fieldRules">
        <el-form-item label="字段名称" prop="fieldName">
          <el-input v-model="newField.fieldName" placeholder="请输入字段名称" />
        </el-form-item>
        <el-form-item label="字段类型" prop="fieldType">
          <el-select v-model="newField.fieldType" placeholder="请选择字段类型" style="width: 100%">
            <el-option v-for="item in fieldTypeOptions"
                       :key="item.value"
                       :label="item.label"
                       :value="item.value" />
          </el-select>
        </el-form-item>
        <template v-if="newField.fieldType !== FieldTypeEnum.Array">
          <el-form-item v-if="newField.fieldType" label="默认值">
            <div v-if="newField.fieldType === FieldTypeEnum.Enum">
              <div class="possible-values-table">
                <el-table :data="possibleValuesArray" 
                          border
                          size="default"
                          max-height="200">
                  <el-table-column label="值" min-width="120">
                    <template #default="scope">
                      <el-input-number v-model="scope.row.value"
                                       :min="0"
                                       placeholder="值"
                                       size="small"
                                       style="width: 100%" />
                    </template>
                  </el-table-column>
                  <el-table-column label="概率(%)" min-width="120">
                    <template #default="scope">
                      <el-input-number v-model="scope.row.probability"
                                       :min="0"
                                       :max="100"
                                       placeholder="概率"
                                       size="small"
                                       style="width: 100%" />
                    </template>
                  </el-table-column>
                  <el-table-column label="操作" width="80" align="center">
                    <template #default="scope">
                      <el-button type="danger"
                                 circle
                                 size="small"
                                 @click="removePossibleValue(scope.$index)">
                        <el-icon>
                          <delete />
                        </el-icon>
                      </el-button>
                    </template>
                  </el-table-column>
                </el-table>
                <div class="table-actions">
                  <el-button type="primary" size="small" @click="addPossibleValue">
                    添加枚举值
                  </el-button>
                </div>
              </div>
            </div>
            <div v-else>
              <!-- 非枚举类型只有一个值 -->
              <el-input v-model="singleDefaultValue" placeholder="默认值" style="width: 100%;" />
            </div>
          </el-form-item>
          <template v-if="newField.fieldType === FieldTypeEnum.Integer || newField.fieldType === FieldTypeEnum.Float">
            <el-form-item label="最小值">
              <el-input-number v-model="newField.minValue"
                               :precision="newField.fieldType === FieldTypeEnum.Float ? 2 : 0" />
            </el-form-item>
            <el-form-item label="最大值">
              <el-input-number v-model="newField.maxValue"
                               :precision="newField.fieldType === FieldTypeEnum.Float ? 2 : 0" />
            </el-form-item>
          </template>
        </template>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="addFieldDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="confirmAddField">确认</el-button>
        </span>
      </template>
    </el-dialog>

    <!-- 编辑字段对话框 -->
    <el-dialog v-model="editFieldDialogVisible"
               title="编辑字段"
               width="500px"
               :close-on-click-modal="false"
               :append-to-body="true">
      <el-form ref="editFieldFormRef"
               :model="currentEditField"
               label-width="80px"
               :rules="fieldRules">
        <el-form-item label="字段名称" prop="fieldName">
          <el-input v-model="currentEditField.fieldName" placeholder="请输入字段名称" />
        </el-form-item>
        <el-form-item label="字段类型" prop="fieldType">
          <el-select v-model="currentEditField.fieldType" placeholder="请选择字段类型" style="width: 100%">
            <el-option v-for="item in fieldTypeOptions"
                       :key="item.value"
                       :label="item.label"
                       :value="item.value" />
          </el-select>
        </el-form-item>
        <template
          v-if="currentEditField.fieldType !== FieldTypeEnum.Array">
          <el-form-item v-if="currentEditField.fieldType" label="默认值">
            <div v-if="currentEditField.fieldType === FieldTypeEnum.Enum">
              <div class="possible-values-table">
                <el-table :data="editPossibleValuesArray" 
                          border 
                          size="small" 
                          max-height="200">
                  <el-table-column label="值" min-width="120">
                    <template #default="scope">
                      <el-input-number v-model="scope.row.value"
                                       :min="0"
                                       placeholder="值"
                                       size="small"
                                       style="width: 100%" />
                    </template>
                  </el-table-column>
                  <el-table-column label="概率(%)" min-width="120">
                    <template #default="scope">
                      <el-input-number v-model="scope.row.probability"
                                       :min="0"
                                       :max="100"
                                       placeholder="概率"
                                       size="small"
                                       style="width: 100%" />
                    </template>
                  </el-table-column>
                  <el-table-column label="操作" width="80" align="center">
                    <template #default="scope">
                      <el-button type="danger"
                                 circle
                                 size="small"
                                 @click="removeEditPossibleValue(scope.$index)">
                        <el-icon>
                          <delete />
                        </el-icon>
                      </el-button>
                    </template>
                  </el-table-column>
                </el-table>
                <div class="table-actions">
                  <el-button type="primary" size="small" @click="addEditPossibleValue">
                    添加可能值
                  </el-button>
                </div>
              </div>
            </div>
            <div v-else>
              <!-- 非枚举类型只有一个值 -->
              <el-input v-model="editSingleDefaultValue" placeholder="默认值" style="width: 100%;" />
            </div>
          </el-form-item>
          <template
            v-if="currentEditField.fieldType === FieldTypeEnum.Integer || currentEditField.fieldType === FieldTypeEnum.Float">
            <el-form-item label="最小值">
              <el-input-number v-model="currentEditField.minValue"
                               :precision="currentEditField.fieldType === FieldTypeEnum.Float ? 2 : 0" />
            </el-form-item>
            <el-form-item label="最大值">
              <el-input-number v-model="currentEditField.maxValue"
                               :precision="currentEditField.fieldType === FieldTypeEnum.Float ? 2 : 0" />
            </el-form-item>
          </template>
        </template>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="editFieldDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="confirmEditField">确认</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>
<script setup lang="ts" name="DataModel">
import { convertToJsonStruct,convertJsonStructToJson } from "@/hooks/processJsonStruct"
import { FieldTypeEnum, JsonStruct, PossibleValue } from "@/types/mqttConfig"
import { open } from "@tauri-apps/plugin-dialog"
import { invoke } from "@tauri-apps/api/core"
import { ElMessage, FormInstance } from "element-plus"
import { Refresh, Plus, Download, Delete } from '@element-plus/icons-vue'
import { useWindowSize } from "@vueuse/core"
const { height } = useWindowSize()

const config = inject<any>("config")
const loading = ref<boolean>(true)
const addFieldDialogVisible = ref(false)
const editFieldDialogVisible = ref(false)
const currentParent = ref<JsonStruct | null>(null)
const currentEditField = ref<JsonStruct>({})
const fieldFormRef = ref<FormInstance>()
const editFieldFormRef = ref<FormInstance>()



const ModelTableHeight = computed(() => {
  return height.value - 400
})

const newField = ref<JsonStruct>({
  fieldName: '',
  fieldType: FieldTypeEnum.String,
  minValue: undefined,
  maxValue: undefined,
  possibleValues: []
})

const fieldRules = {
  fieldName: [
    { required: true, message: '请输入字段名称', trigger: 'blur' },
    { pattern: /^[a-zA-Z][a-zA-Z0-9_]*$/, message: '字段名称只能包含字母、数字和下划线，且必须以字母开头', trigger: 'blur' }
  ],
  fieldType: [
    { required: true, message: '请选择字段类型', trigger: 'change' }
  ]
}

const fieldTypeOptions = [
  { value: FieldTypeEnum.String, label: '字符串(String)' },
  { value: FieldTypeEnum.Integer, label: '整数(Integer)' },
  { value: FieldTypeEnum.Float, label: '浮点数(Float)' },
  { value: FieldTypeEnum.Boolean, label: '布尔值(Boolean)' },
  { value: FieldTypeEnum.Enum, label: '枚举(Array)' },
  { value: FieldTypeEnum.Array, label: '数组(Array)' },
  { value: FieldTypeEnum.DateTime, label: '日期时间(DateTime)' },
  { value: FieldTypeEnum.Timestamp, label: '时间戳(Timestamp)' }
]

const possibleValuesArray = ref<PossibleValue[]>([])
const editPossibleValuesArray = ref<PossibleValue[]>([])
const singleDefaultValue = ref<any>(undefined)
const editSingleDefaultValue = ref<any>(undefined)

// 根据字段类型转换值
const convertValueByFieldType = (value: any, fieldType: FieldTypeEnum): any => {
  switch (fieldType) {
    case FieldTypeEnum.Integer:
      return Number.isInteger(Number(value)) ? Number(value) : Math.floor(Number(value))
    case FieldTypeEnum.Float:
    case FieldTypeEnum.Timestamp:
      return Number(value)
    case FieldTypeEnum.Boolean:
      return Boolean(value)
    case FieldTypeEnum.String:
    case FieldTypeEnum.DateTime:
      return String(value)
    case FieldTypeEnum.Enum:
    default:
      return value
  }
}

// 转换possibleValues数组中的值类型
const convertPossibleValuesType = (possibleValues: PossibleValue[], fieldType: FieldTypeEnum): PossibleValue[] => {
  return possibleValues.map(item => ({
    value: convertValueByFieldType(item.value, fieldType),
    probability: item.probability
  }))
}

// 添加可能值（创建表单）
const addPossibleValue = () => {
  possibleValuesArray.value.push({ value: 0, probability: 0 })
}

// 移除可能值（创建表单）
const removePossibleValue = (index: number) => {
  possibleValuesArray.value.splice(index, 1)
}

// 添加可能值（编辑表单）
const addEditPossibleValue = () => {
  editPossibleValuesArray.value.push({ value: 0, probability: 0 })
}

// 移除可能值（编辑表单）
const removeEditPossibleValue = (index: number) => {
  editPossibleValuesArray.value.splice(index, 1)
}

// 获取字段类型名称
const getFieldTypeName = (type: FieldTypeEnum): string => {
  const option = fieldTypeOptions.find(opt => opt.value === type)
  return option ? option.label : '未知类型'
}


// 刷新数据结构
const refreshStructure = () => {
  loading.value = true
  let parsedData = {}
  try {
    parsedData = JSON.parse(config.value.sendData || '{}')
  } catch (e) {
    console.error(e)
    ElMessage.error('JSON格式错误，无法解析')
  }
  config.value.fieldStruct = convertToJsonStruct(parsedData, [])
  updateJsonFromStruct()
  loading.value = false
  ElMessage.success('数据结构已刷新')
}

// 更新JSON数据
const updateJsonFromStruct = () => {
  config.value.sendData = JSON.stringify(convertJsonStructToJson(config.value.fieldStruct), null, 2)
}



// 显示添加字段对话框
const showAddFieldDialog = () => {
  newField.value = {
    fieldName: '',
    fieldType: FieldTypeEnum.String,
    minValue: undefined,
    maxValue: undefined,
    possibleValues: []
  }
  
  possibleValuesArray.value = []
  singleDefaultValue.value = undefined

  currentParent.value = null
  addFieldDialogVisible.value = true
}

// 显示编辑字段对话框
const showEditFieldDialog = (row: JsonStruct) => {
  currentEditField.value = JSON.parse(JSON.stringify(row))
  
  // 初始化编辑表单中的可能值数组
  if (currentEditField.value.fieldType === FieldTypeEnum.Enum) {
    editPossibleValuesArray.value = [...(currentEditField.value.possibleValues || [])]
  } else if (currentEditField.value.fieldType !== FieldTypeEnum.Array && 
             currentEditField.value.possibleValues?.length) {
    editSingleDefaultValue.value = currentEditField.value.possibleValues[0]?.value
  } else {
    editSingleDefaultValue.value = undefined
  }
  
  editFieldDialogVisible.value = true
}

// 确认添加字段前处理possibleValues
const confirmAddField = () => {
  fieldFormRef.value?.validate((valid) => {
    if (valid) {
      const fieldToAdd = { ...newField.value }

      // 如果是枚举类型，设置possibleValues为数组
      if (fieldToAdd.fieldType === FieldTypeEnum.Enum) {
        fieldToAdd.possibleValues = convertPossibleValuesType([...possibleValuesArray.value], fieldToAdd.fieldType)
      } else if (fieldToAdd.fieldType !== FieldTypeEnum.Array) {
        // 非enum类型的probability固定为100%
        fieldToAdd.possibleValues = singleDefaultValue.value !== undefined 
          ? [{ value: convertValueByFieldType(singleDefaultValue.value, fieldToAdd.fieldType), probability: 100 }] 
          : []
      }

      // 添加为根级字段
      config.value.fieldStruct.push(fieldToAdd)

      addFieldDialogVisible.value = false
      updateJsonFromStruct()
      ElMessage.success('字段添加成功')
    }
  })
}

// 确认编辑字段前处理possibleValues
const confirmEditField = () => {
  editFieldFormRef.value?.validate((valid) => {
    if (valid) {
      
      // 如果是枚举类型，设置possibleValues为数组
      if (currentEditField.value.fieldType === FieldTypeEnum.Enum) {
        currentEditField.value.possibleValues = convertPossibleValuesType([...editPossibleValuesArray.value], currentEditField.value.fieldType)
      } else if (currentEditField.value.fieldType !== FieldTypeEnum.Array) {
        // 非enum类型的probability固定为1
        currentEditField.value.possibleValues = editSingleDefaultValue.value !== undefined 
          ? [{ value: convertValueByFieldType(editSingleDefaultValue.value, currentEditField.value.fieldType), probability: 100 }] 
          : []
      }

      // 找到字段并更新
      const updateField = (arr: JsonStruct[], fieldId: string) => {
        for (let i = 0; i < arr.length; i++) {
          if (arr[i].fieldName === fieldId) {
            // 更新字段属性
            arr[i] = { ...currentEditField.value }
            return true
          }

        }
        return false
      }

      updateField(config.value.fieldStruct, currentEditField.value.fieldName)
      editFieldDialogVisible.value = false
      updateJsonFromStruct()
      ElMessage.success('字段更新成功')
    }
  })
}

// 导入模板
const importTemplate = async () => {
  try {
    const filePath = await open({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }]
    }) as string

    if (filePath) {
      const templateContent = await invoke('read_file', { path: filePath })
      try {
        const parsedTemplate = JSON.parse(templateContent as string)
        config.value.fieldStruct = convertToJsonStruct(parsedTemplate, [])
        config.value.sendData = JSON.stringify(parsedTemplate, null, 2)
        ElMessage.success('模板导入成功')
      } catch (e) {
        console.error(e)
        ElMessage.error('模板文件格式错误')
      }
    }
  } catch (error) {
    ElMessage.error(`导入失败: ${error}`)
  }
}

watch(() => config.value.fieldStruct, (newVal) => {
  if (newVal === undefined) return
  config.value.fieldStruct = newVal
  updateJsonFromStruct()
}, { deep: true })

onMounted(() => {
  loading.value = true
  let parsedData = {}
  try {
    parsedData = JSON.parse(config.value.sendData || '{}')
  } catch (e) {
    console.error(e)
    ElMessage.error('JSON格式错误，无法解析')
  }
  config.value.fieldStruct = convertToJsonStruct(parsedData, config.value.fieldStruct || [])
  loading.value = false
})

// 在处理编辑对话框时初始化默认值
watch(() => currentEditField.value.fieldType, (newType) => {
  // 清空编辑表单的默认值数据
  editPossibleValuesArray.value = []
  editSingleDefaultValue.value = undefined
  
  // 清空数值范围
  currentEditField.value.minValue = undefined
  currentEditField.value.maxValue = undefined
  currentEditField.value.possibleValues = []
  
  // 如果新类型不是枚举类型且原来有默认值，则尝试恢复第一个值
  if (newType !== FieldTypeEnum.Enum && currentEditField.value.possibleValues?.length) {
    editSingleDefaultValue.value = currentEditField.value.possibleValues[0]?.value
  }
})

// 在处理新建字段时，字段类型切换时清空默认值表单
watch(() => newField.value.fieldType, (_newType) => {
  // 清空所有默认值相关的表单数据
  possibleValuesArray.value = []
  singleDefaultValue.value = undefined
  
  // 清空数值范围
  newField.value.minValue = undefined
  newField.value.maxValue = undefined
  newField.value.possibleValues = []
})
</script>
<style lang="scss" scoped>
.data-model {
  .table-container {
    margin-bottom: 20px;
  }

  .tips {
    margin-top: 20px;

    .tips-content {
      p {
        margin: 8px 0;
        line-height: 1.5;
      }

      .el-tag {
        margin-right: 8px;
      }
    }
  }
}
</style>
