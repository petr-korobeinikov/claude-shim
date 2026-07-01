import { defineConfig } from 'vitepress'
import llmstxt from 'vitepress-plugin-llms'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: 'claude-shim',
  description: 'Claude Code profile manager. Combat-proven. Automagic.',
  base: '/claude-shim/',
  vite: {
    plugins: [llmstxt({ domain: 'https://petr-korobeinikov.github.io' })],
  },
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    search: {
      provider: 'local',
    },
    nav: [
      { text: 'Quick Start', link: '/guide/quick-start' },
    ],
    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Quick Start', link: '/guide/quick-start' },
          { text: 'Installation', link: '/guide/installation' },
          { text: 'Prompt indicator', link: '/guide/prompt-indicator' },
          { text: 'statusLine indicator', link: '/guide/statusline' },
          { text: 'Profiles', link: '/guide/profiles' },
          { text: 'Profile resolution', link: '/guide/resolution' },
          { text: 'Effort level', link: '/guide/effort' },
          { text: 'Migration', link: '/guide/migration' },
        ],
      },
      { text: 'Contributing', link: '/contributing' },
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/petr-korobeinikov/claude-shim' },
    ],
    footer: {
      message: 'Released under the MIT License.',
    },
  },
})
