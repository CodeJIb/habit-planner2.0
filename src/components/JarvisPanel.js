import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { Mic, Plus, Settings, Trash2 } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { ConfirmDialog } from './ConfirmDialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue, } from './ui/select';
const SUGGESTIONS = ['Plan my day', 'Check my streaks', 'How much XP to level up?'];
/** Right-hand assistant panel. Talks to the Rust AI service (Phase 5). */
export function JarvisPanel({ messages, conversations, activeConversationId, busy, streaming, pendingUser, settings, onSend, onSelectConversation, onNewConversation, onDeleteConversation, onSaveSettings, }) {
    const [draft, setDraft] = useState('');
    const [showSettings, setShowSettings] = useState(false);
    const [apiKey, setApiKey] = useState('');
    /** Local overrides while editing; each falls back to the saved value. */
    const [modelDraft, setModelDraft] = useState(null);
    const [baseUrlDraft, setBaseUrlDraft] = useState(null);
    const [pendingDelete, setPendingDelete] = useState(null);
    const logRef = useRef(null);
    const model = modelDraft ?? settings?.model ?? '';
    const baseUrl = baseUrlDraft ?? settings?.baseUrl ?? '';
    const usesFreeBackend = baseUrl.trim().length > 0;
    // Follow the conversation as it grows.
    useEffect(() => {
        const log = logRef.current;
        if (log)
            log.scrollTop = log.scrollHeight;
    }, [messages, busy, streaming, pendingUser]);
    const needsKey = settings !== null && !usesFreeBackend && !settings.apiKeySet;
    const canSend = !busy && draft.trim().length > 0;
    function send(text) {
        const trimmed = text.trim();
        if (!trimmed || busy)
            return;
        onSend(trimmed);
        setDraft('');
    }
    function saveSettings() {
        // An empty key field means "leave the stored key alone".
        onSaveSettings({
            apiKey: apiKey.trim() ? apiKey.trim() : null,
            baseUrl: baseUrl.trim(),
            model: model.trim(),
        });
        setApiKey('');
        setModelDraft(null);
        setBaseUrlDraft(null);
        setShowSettings(false);
    }
    function clearKey() {
        onSaveSettings({ apiKey: '' });
        setApiKey('');
    }
    return (_jsxs("aside", { className: "assistant", "aria-label": "JARVIS assistant", children: [_jsxs("header", { children: [_jsx("div", { className: "orb", "aria-hidden": true }), _jsxs("div", { children: [_jsx("b", { children: "JARVIS" }), _jsx("span", { children: settings?.model
                                    ? `${usesFreeBackend ? 'Custom' : 'Claude'} · ${settings.model}`
                                    : 'Assistant' })] }), _jsx(Button, { type: "button", variant: "ghost", size: "icon", "aria-label": "Assistant settings", "aria-expanded": showSettings, onClick: () => setShowSettings((open) => !open), style: { marginLeft: 'auto' }, children: _jsx(Settings, { "aria-hidden": true }) })] }), showSettings && (_jsxs("div", { className: "assistant-settings", role: "group", "aria-label": "Assistant settings", children: [_jsxs("label", { children: [_jsx("span", { children: "Base URL (free / OpenAI-compatible \u2014 leave empty for Anthropic)" }), _jsx(Input, { value: baseUrl, onChange: (e) => setBaseUrlDraft(e.target.value), placeholder: "http://localhost:11434/v1", "aria-label": "Base URL" })] }), _jsxs("label", { children: [_jsxs("span", { children: ["API key ", usesFreeBackend ? '(optional)' : ''] }), _jsx(Input, { type: "password", value: apiKey, onChange: (e) => setApiKey(e.target.value), placeholder: settings?.apiKeySet ? 'Key saved — type to replace' : 'sk-ant-…', "aria-label": "API key" })] }), _jsxs("label", { children: [_jsx("span", { children: "Model" }), _jsx(Input, { value: model, onChange: (e) => setModelDraft(e.target.value), placeholder: usesFreeBackend ? 'openai/gpt-oss-20b' : 'claude-sonnet-5-5', "aria-label": "Model id" })] }), _jsxs("div", { className: "row", children: [_jsx(Button, { type: "button", size: "sm", onClick: saveSettings, children: "Save" }), settings?.apiKeySet && (_jsx(Button, { type: "button", size: "sm", variant: "outline", onClick: clearKey, children: "Remove key" }))] }), _jsx("p", { children: "Leave the key empty to use the hosted proxy. Or set a Base URL to any OpenAI-compatible server (Groq, Ollama) and supply your own key. Settings stay on this device." })] })), conversations.length > 0 && (_jsxs("div", { className: "assistant-threads", children: [_jsxs(Select, { value: activeConversationId === null ? undefined : String(activeConversationId), onValueChange: (value) => onSelectConversation(Number(value)), children: [_jsx(SelectTrigger, { size: "sm", "aria-label": "Conversation", style: { flex: 1, minWidth: 0 }, children: _jsx(SelectValue, { placeholder: "New chat" }) }), _jsx(SelectContent, { children: conversations.map((conversation) => (_jsx(SelectItem, { value: String(conversation.id), children: conversation.title ?? `Chat ${conversation.id}` }, conversation.id))) })] }), _jsx(Button, { type: "button", variant: "outline", size: "icon", "aria-label": "New chat", onClick: onNewConversation, children: _jsx(Plus, { "aria-hidden": true }) }), activeConversationId !== null && (_jsx(Button, { type: "button", variant: "outline", size: "icon", "aria-label": "Delete chat", onClick: () => setPendingDelete(conversations.find((c) => c.id === activeConversationId) ?? null), children: _jsx(Trash2, { "aria-hidden": true }) }))] })), _jsxs("div", { className: "log", "aria-live": "polite", ref: logRef, children: [messages.length === 0 && !busy && !streaming && !pendingUser ? (_jsx("p", { style: { color: 'var(--color-muted)' }, children: needsKey
                            ? 'Add an Anthropic API key, or set a free base URL, to switch me on.'
                            : 'Hello. Ask me to plan your day, check your progress or add a task.' })) : (messages.map((message) => (_jsx("p", { className: `msg ${message.role}`, children: message.content }, message.id)))), pendingUser !== null && _jsx("p", { className: "msg user", children: pendingUser }), streaming ? (_jsx("p", { className: "msg assistant streaming", children: streaming })) : (busy && _jsx("p", { className: "msg assistant thinking", children: "JARVIS is thinking\u2026" })), needsKey && (_jsx(Button, { type: "button", variant: "outline", size: "sm", onClick: () => setShowSettings(true), children: "Configure assistant" }))] }), _jsxs("footer", { children: [_jsx("div", { className: "chips", style: { display: 'flex', gap: '0.375rem', flexWrap: 'wrap' }, children: SUGGESTIONS.map((suggestion) => (_jsx(Button, { variant: "outline", size: "sm", disabled: busy, onClick: () => send(suggestion), children: suggestion }, suggestion))) }), _jsxs("form", { onSubmit: (e) => {
                            e.preventDefault();
                            send(draft);
                        }, children: [_jsx("input", { value: draft, onChange: (e) => setDraft(e.target.value), placeholder: "Ask JARVIS\u2026", "aria-label": "Message JARVIS", disabled: busy }), _jsx(Button, { type: "button", variant: "outline", size: "icon", "aria-label": "Voice input (coming soon)", disabled: true, children: _jsx(Mic, { "aria-hidden": true }) }), _jsx(Button, { type: "submit", disabled: !canSend, children: "Send" })] })] }), pendingDelete && (_jsx(ConfirmDialog, { title: "Delete this chat?", body: "The conversation and its messages will be permanently deleted.", onCancel: () => setPendingDelete(null), onConfirm: () => {
                    onDeleteConversation(pendingDelete.id);
                    setPendingDelete(null);
                } }))] }));
}
