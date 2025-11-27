import React, { useState, useEffect } from 'react';
import { apiClient } from '../../api/client';
import type { PromptFile } from '../../types/api.types';

interface PromptManagerProps {
  className?: string;
}

export const PromptManager: React.FC<PromptManagerProps> = ({ className }) => {
  const [prompts, setPrompts] = useState<PromptFile[]>([]);
  const [selectedPrompt, setSelectedPrompt] = useState<PromptFile | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const [showNewPrompt, setShowNewPrompt] = useState(false);
  const [newPromptName, setNewPromptName] = useState('');
  const [newPromptContent, setNewPromptContent] = useState('');
  const [editingContent, setEditingContent] = useState('');
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [missingPrompts, setMissingPrompts] = useState(false);

  // 加载所有prompts
  const loadPrompts = async () => {
    try {
      setIsLoading(true);
      const data = await apiClient.getPrompts();
      setPrompts(data);
      setMissingPrompts(data.length === 0);
      if (data.length === 0) {
        setMessage({
          type: 'error',
          text: '未检测到提示词，请先创建或导入，否则后端将拒绝任务',
        });
        setShowNewPrompt(true);
      }
    } catch (error) {
      console.error('Failed to load prompts:', error);
      setMissingPrompts(true);
      setMessage({ type: 'error', text: '加载提示词失败，可能未配置 ban_prompts 目录' });
      setShowNewPrompt(true);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadPrompts();
  }, []);

  // 过滤prompts
  const filteredPrompts = prompts.filter(prompt =>
    prompt.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    prompt.content.toLowerCase().includes(searchTerm.toLowerCase())
  );

  // 选择prompt进行编辑
  const handleSelectPrompt = async (prompt: PromptFile) => {
    try {
      const fullPrompt = await apiClient.getPrompt(prompt.name);
      setSelectedPrompt(fullPrompt);
      setEditingContent(fullPrompt.content);
      setShowNewPrompt(false);
    } catch (error) {
      console.error('Failed to load prompt:', error);
      setMessage({ type: 'error', text: '加载提示词内容失败' });
    }
  };

  // 创建新prompt
  const handleCreateNew = () => {
    setSelectedPrompt(null);
    setNewPromptName('');
    setNewPromptContent('');
    setShowNewPrompt(true);
  };

  // 保存新prompt
  const handleSaveNew = async () => {
    if (!newPromptName.trim()) {
      setMessage({ type: 'error', text: '请输入提示词名称' });
      return;
    }

    try {
      setIsSaving(true);
      const saved = await apiClient.savePrompt(newPromptName.trim(), newPromptContent);
      setPrompts(prev => [...prev.filter(p => p.name !== saved.name), saved]);
      setShowNewPrompt(false);
      setNewPromptName('');
      setNewPromptContent('');
      setMessage({ type: 'success', text: '提示词创建成功' });
    } catch (error) {
      console.error('Failed to save prompt:', error);
      setMessage({ type: 'error', text: '保存提示词失败' });
    } finally {
      setIsSaving(false);
    }
  };

  // 更新现有prompt
  const handleUpdateExisting = async () => {
    if (!selectedPrompt) return;

    try {
      setIsSaving(true);
      const updated = await apiClient.savePrompt(selectedPrompt.name, editingContent);
      setPrompts(prev => [...prev.filter(p => p.name !== updated.name), updated]);
      setSelectedPrompt(updated);
      setMessage({ type: 'success', text: '提示词更新成功' });
    } catch (error) {
      console.error('Failed to update prompt:', error);
      setMessage({ type: 'error', text: '更新提示词失败' });
    } finally {
      setIsSaving(false);
    }
  };

  // 删除prompt
  const handleDelete = async (promptName: string) => {
    if (!confirm(`确定要删除提示词 "${promptName}" 吗？`)) {
      return;
    }

    try {
      await apiClient.deletePrompt(promptName);
      setPrompts(prev => prev.filter(p => p.name !== promptName));
      if (selectedPrompt?.name === promptName) {
        setSelectedPrompt(null);
        setEditingContent('');
      }
      setMessage({ type: 'success', text: '提示词删除成功' });
    } catch (error) {
      console.error('Failed to delete prompt:', error);
      setMessage({ type: 'error', text: '删除提示词失败' });
    }
  };

  // 导出prompt
  const handleExport = (prompt: PromptFile) => {
    const blob = new Blob([prompt.content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${prompt.name}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  // 导入prompt
  const handleImport = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (e) => {
      const content = e.target?.result as string;
      const name = file.name.replace('.txt', '');

      try {
        setIsSaving(true);
        const saved = await apiClient.savePrompt(name, content);
        setPrompts(prev => [...prev.filter(p => p.name !== saved.name), saved]);
        setMessage({ type: 'success', text: '提示词导入成功' });
      } catch (error) {
        console.error('Failed to import prompt:', error);
        setMessage({ type: 'error', text: '导入提示词失败' });
      } finally {
        setIsSaving(false);
      }
    };
    reader.readAsText(file);
    event.target.value = '';
  };

  // 清除消息
  useEffect(() => {
    if (message) {
      const timer = setTimeout(() => setMessage(null), 3000);
      return () => clearTimeout(timer);
    }
  }, [message]);

  return (
    <div className={`p-6 ${className}`}>
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">提示词管理</h2>
        <div className="flex gap-2">
          <label className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 cursor-pointer">
            导入提示词
            <input
              type="file"
              accept=".txt"
              onChange={handleImport}
              className="hidden"
            />
          </label>
          <button
            onClick={handleCreateNew}
            className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
          >
            新建提示词
          </button>
        </div>
      </div>

      {message && (
        <div className={`mb-4 p-3 rounded ${
          message.type === 'success' ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'
        }`}>
          {message.text}
        </div>
      )}
      {missingPrompts && (
        <div className="mb-4 p-3 rounded bg-red-50 text-red-700 border border-red-200">
          未找到任何提示词。请点击右上角“新建提示词”或“导入提示词”添加至少一条，以便后端正常处理任务。
        </div>
      )}

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* 左侧：列表 */}
        <div>
          <input
            type="text"
            placeholder="搜索提示词..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full px-3 py-2 border rounded mb-4"
          />

          {isLoading ? (
            <div className="text-center py-8">加载中...</div>
          ) : (
            <div className="space-y-2 max-h-96 overflow-y-auto">
              {filteredPrompts.map((prompt) => (
                <div
                  key={prompt.name}
                  className={`p-3 border rounded cursor-pointer hover:bg-gray-50 ${
                    selectedPrompt?.name === prompt.name ? 'bg-blue-50 border-blue-300' : ''
                  }`}
                  onClick={() => handleSelectPrompt(prompt)}
                >
                  <div className="flex justify-between items-start">
                    <div className="flex-1">
                      <h3 className="font-semibold">{prompt.name}</h3>
                      <p className="text-sm text-gray-600 truncate">{prompt.content}</p>
                      <div className="text-xs text-gray-400 mt-1">
                        {new Date(prompt.modified_at).toLocaleString()} · {prompt.size} 字符
                      </div>
                    </div>
                    <div className="flex gap-1 ml-2">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleExport(prompt);
                        }}
                        className="p-1 text-gray-500 hover:text-blue-600"
                        title="导出"
                      >
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                        </svg>
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDelete(prompt.name);
                        }}
                        className="p-1 text-gray-500 hover:text-red-600"
                        title="删除"
                      >
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              ))}
              {filteredPrompts.length === 0 && (
                <div className="text-center py-8 text-gray-500">
                  {searchTerm ? '没有找到匹配的提示词' : '暂无提示词'}
                </div>
              )}
            </div>
          )}
        </div>

        {/* 右侧：编辑器 */}
        <div>
          {showNewPrompt ? (
            <div>
              <h3 className="text-lg font-semibold mb-3">新建提示词</h3>
              <input
                type="text"
                placeholder="提示词名称"
                value={newPromptName}
                onChange={(e) => setNewPromptName(e.target.value)}
                className="w-full px-3 py-2 border rounded mb-3"
              />
              <textarea
                placeholder="输入提示词内容..."
                value={newPromptContent}
                onChange={(e) => setNewPromptContent(e.target.value)}
                className="w-full h-64 px-3 py-2 border rounded resize-none"
              />
              <div className="flex gap-2 mt-3">
                <button
                  onClick={handleSaveNew}
                  disabled={isSaving}
                  className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
                >
                  {isSaving ? '保存中...' : '保存'}
                </button>
                <button
                  onClick={() => setShowNewPrompt(false)}
                  className="px-4 py-2 bg-gray-300 text-gray-700 rounded hover:bg-gray-400"
                >
                  取消
                </button>
              </div>
            </div>
          ) : selectedPrompt ? (
            <div>
              <h3 className="text-lg font-semibold mb-3">编辑: {selectedPrompt.name}</h3>
              <div className="text-sm text-gray-500 mb-3">
                创建于: {new Date(selectedPrompt.created_at).toLocaleString()} |
                修改于: {new Date(selectedPrompt.modified_at).toLocaleString()} |
                {selectedPrompt.size} 字符
              </div>
              <textarea
                value={editingContent}
                onChange={(e) => setEditingContent(e.target.value)}
                className="w-full h-64 px-3 py-2 border rounded resize-none"
              />
              <div className="flex gap-2 mt-3">
                <button
                  onClick={handleUpdateExisting}
                  disabled={isSaving}
                  className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
                >
                  {isSaving ? '更新中...' : '更新'}
                </button>
                <button
                  onClick={() => handleExport(selectedPrompt)}
                  className="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700"
                >
                  导出
                </button>
                <button
                  onClick={() => handleDelete(selectedPrompt.name)}
                  className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
                >
                  删除
                </button>
              </div>
            </div>
          ) : (
            <div className="text-center py-16 text-gray-500">
              选择一个提示词进行编辑，或创建新的提示词
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
