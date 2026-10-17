import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
/** Screen for features that are planned but not implemented yet (Phases 3+). */
export function PlaceholderPage({ title, body }) {
    return (_jsxs("div", { children: [_jsx("h1", { children: title }), _jsx("p", { className: "sub", children: body }), _jsx("div", { style: {
                    border: '1px dashed var(--color-border)',
                    borderRadius: 'var(--radius-lg)',
                    padding: '3rem 1.5rem',
                    textAlign: 'center',
                    color: 'var(--color-muted)',
                }, children: "This screen arrives in a later phase. Until then, use the pages above." })] }));
}
