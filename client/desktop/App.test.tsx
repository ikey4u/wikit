import { expect, open, test } from "nsh/test";

test("native navigation works at phone width", async () => {
  const app = await open("./App.tsx", { width: 390, height: 844 });
  await app.waitFor("你的随身词典");
  expect(app.texts()).toContain("Wikit");
  await app.press("label:词典");
  await app.waitFor("词典与设置");
  expect(app.shows("导入到设备")).toBe(true);
  await app.press("label:笔记");
  await app.waitFor("学习笔记");
  await app.press("label:查词");
  await app.press("label:查询");
  await app.waitFor("请输入单词，并先添加、选择一本词典。");
});

test("desktop sidebar opens favorites and preview", async () => {
  const app = await open("./App.tsx", { width: 1100, height: 760 });
  await app.waitFor("你的随身词典");
  await app.press("label:收藏");
  await app.waitFor("收藏的单词");
  await app.press("label:历史");
  await app.waitFor("最近查询");
  await app.press("label:预览");
  await app.waitFor("词典页面预览");
  expect(app.shows("刷新预览")).toBe(true);
});
