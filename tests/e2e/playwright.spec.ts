import { test, expect } from '@playwright/test';

// 端到端冒烟：登录 -> 提交 Cookie -> 查看队列 -> 修改并发
// 运行前：
// 1) 后端前端同时启动（前端应代理到 8484）
// 2) 设置 ADMIN_PASSWORD 环境变量或手动填写下面的密码
// 3) `npx playwright test tests/e2e/playwright.spec.ts`

const ADMIN_PASSWORD = process.env.ADMIN_PASSWORD || 'changeme';
const BASE = process.env.APP_BASE || 'http://127.0.0.1:8484';

const SAMPLE_COOKIE = `sk-ant-sid01-${'A'.repeat(86)}-ZZZZ`;

test('login, submit cookie, view stats, update concurrency', async ({ page }) => {
  await page.goto(BASE);

  // 登录
  await page.getByPlaceholder('请输入密码').fill(ADMIN_PASSWORD);
  await page.getByRole('button', { name: '登录' }).click();
  await expect(page.getByText('控制台概览')).toBeVisible({ timeout: 5000 });

  // 跳转到 Cookie 管理并提交
  await page.getByRole('button', { name: '添加Cookie' }).click();
  await page.getByLabel('Cookie 列表（每行一个）').fill(SAMPLE_COOKIE);
  await page.getByRole('button', { name: /提交 1 个 Cookie/ }).click();
  await expect(page.getByText(/成功提交 1 个 Cookie/)).toBeVisible();

  // 查看统计
  await page.getByRole('button', { name: '查看统计' }).click();
  await expect(page.getByText('系统统计')).toBeVisible();

  // 修改并发（配置页面）
  await page.getByRole('button', { name: '配置系统' }).click();
  await page.getByLabel('并发工作线程数').fill('3');
  await page.getByRole('button', { name: '保存配置' }).click();
  await expect(page.getByText('配置保存成功')).toBeVisible();
});
