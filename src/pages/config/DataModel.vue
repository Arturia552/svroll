<template>
  <div v-loading="loading">
    <el-table
      :data="config.fieldStruct"
      style="width: 100%"
      row-key="fieldName"
      :tree-props="{ children: 'children' }"
    >
      <el-table-column
        prop="fieldName"
        label="键值"
        min-width="20%"
      ></el-table-column>
      <el-table-column
        prop="fieldType"
        label="数据类型"
        min-width="20%"
      ></el-table-column>
      <el-table-column prop="minValue" label="最小值" min-width="20%">
        <template #default="scope">
          <onClickOutside
            v-if="scope.row.editing1"
            @trigger="scope.row.editing1 = false"
          >
            <el-input
              v-if="scope.row.editing1"
              v-model="scope.row.minValue"
              @blur="
                blurInput(scope.row, scope.row.minValue, 'minValue', 'editing1')
              "
            ></el-input>
          </onClickOutside>
          <span
            v-else
            style="display: block; width: 100%"
            @dblclick="scope.row.editing1 = true"
            >{{ scope.row.minValue ?? "--" }}</span
          >
        </template>
      </el-table-column>
      <el-table-column prop="maxValue" label="最大值" min-width="20%">
        <template #default="scope">
          <onClickOutside
            v-if="scope.row.editing2"
            @trigger="scope.row.editing2 = false"
          >
            <el-input
              v-if="scope.row.editing2"
              v-model="scope.row.maxValue"
              @blur="
                blurInput(scope.row, scope.row.maxValue, 'maxValue', 'editing2')
              "
            ></el-input>
          </onClickOutside>
          <span
            style="display: block; width: 100%"
            v-else
            @dblclick="scope.row.editing2 = true"
            >{{ scope.row.maxValue ?? "--" }}</span
          >
        </template>
      </el-table-column>
      <el-table-column prop="possibleValues" label="有效值" min-width="20%">
        <template #default="scope">
          <onClickOutside
            v-if="
              scope.row.editing3 && scope.row.fieldType !== FieldTypeEnum.Object
            "
            @trigger="scope.row.editing3 = false"
          >
            <el-input
              v-if="scope.row.editing3"
              v-model="scope.row.possibleValues"
              @blur="
                blurInput(
                  scope.row,
                  scope.row.possibleValues,
                  'possibleValues',
                  'editing3'
                )
              "
            ></el-input>
          </onClickOutside>
          <span
            style="display: block; width: 100%"
            v-else
            @dblclick="scope.row.editing3 = true"
            >{{ scope.row.possibleValues ?? "--" }}</span
          >
        </template></el-table-column
      >
    </el-table>
  </div>
</template>
<script setup lang="ts" name="DataModel">
import { convertToJsonStruct } from "@/hooks/processJsonStruct";
import { FieldTypeEnum, JsonStruct, MqttConfig } from "@/types/mqttConfig";
import { OnClickOutside } from "@vueuse/components";

const config = inject<any>("config")
const loading = ref<boolean>(true);

const convertJsonStructToJson = (jsonStructArray: JsonStruct[]): object => {
  const result: any = {};
  jsonStructArray.forEach((item) => {
    if (item.fieldType === FieldTypeEnum.Object) {
      result[item.fieldName] = convertJsonStructToJson(item.children);
    } else {
      result[item.fieldName] = item.possibleValues;
    }
  });
  return result;
};

const blurInput = (
  row: JsonStruct,
  val: any,
  valueKey: string,
  editKey: string
) => {
  row[editKey] = false;
  if (row.fieldType === FieldTypeEnum.Object) return;
  else if (row.fieldType === FieldTypeEnum.String) {
    row[valueKey] = val;
  } else if (row.fieldType === FieldTypeEnum.Integer) {
    row[valueKey] = parseInt(val);
  } else if (row.fieldType === FieldTypeEnum.Float) {
    row[valueKey] = parseFloat(val);
  } else if (row.fieldType === FieldTypeEnum.Boolean) {
    row[valueKey] = val === "true";
  }
};

watch(() => config.value.fieldStruct, (newVal) => {
  if (newVal === undefined) return;
  config.value.fieldStruct = newVal
  config.value.sendData = JSON.stringify(convertJsonStructToJson(newVal));
},{deep: true});

onMounted(() => {
  loading.value = true;
  let parsedData = {};
  try {
    parsedData = JSON.parse(config.value.sendData);
  } catch (e) {}
  config.value.fieldStruct = convertToJsonStruct(parsedData, config.value.fieldStruct);
  loading.value = false;
});
</script>
<style lang="scss" scoped></style>
