import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { sharedMessages } from '@klima/core'
import { I18nProvider } from '@klima/core/ui'
import App from './App'
import { webMessages } from './i18n/messages'
import './index.css'

const CATALOGS = [sharedMessages, webMessages]

const container = document.getElementById('root')
if (!container) throw new Error('Élément #root introuvable dans index.html')

createRoot(container).render(
  <StrictMode>
    <I18nProvider catalogs={CATALOGS}>
      <App />
    </I18nProvider>
  </StrictMode>,
)
