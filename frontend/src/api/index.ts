// frontend/src/api/index.ts
// Re-export from unified client for backward compatibility
export {
  apiClient,
  getVersion,
  validateAuthToken as validateAuthToken,
  postCookie,
  postMultipleCookies,
  getCookieStatus,
  checkCookieStatus,
  deleteCookie,
  adminAction,
  clearPendingQueue,
  clearBannedQueue,
  pauseAllWorkers,
  resumeAllWorkers,
  emergencyStop,
  getSystemStatus,
  fetchConfig,
  saveConfig,
  resetConfig,
  type ApiClient,
} from "./client";
