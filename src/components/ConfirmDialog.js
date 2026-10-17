import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useEffect } from 'react';
import { Button } from './ui/button';
import { Card } from './ui/card';
/** Small confirm dialog used before a destructive action. */
export function ConfirmDialog({ title, body, confirmLabel = 'Delete', onConfirm, onCancel }) {
    // Escape cancels the dialog.
    useEffect(() => {
        const onKey = (e) => {
            if (e.key === 'Escape')
                onCancel();
        };
        window.addEventListener('keydown', onKey);
        return () => window.removeEventListener('keydown', onKey);
    }, [onCancel]);
    return (_jsx("div", { role: "dialog", "aria-modal": "true", "aria-label": title, style: {
            position: 'fixed',
            inset: 0,
            zIndex: 40,
            display: 'grid',
            placeItems: 'center',
            background: 'rgba(0, 0, 0, 0.4)',
        }, onClick: onCancel, children: _jsxs(Card, { style: { maxWidth: '26rem', width: '90vw' }, onClick: (e) => e.stopPropagation(), children: [_jsx("h2", { style: { marginTop: 0 }, children: title }), _jsx("p", { style: { color: 'var(--color-muted)' }, children: body }), _jsxs("div", { style: { display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }, children: [_jsx(Button, { variant: "outline", onClick: onCancel, autoFocus: true, children: "Cancel" }), _jsx(Button, { variant: "destructive", onClick: onConfirm, children: confirmLabel })] })] }) }));
}
