import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: 'claude-shim',
  description: 'Profile manager for Claude Code',
  base: '/claude-shim/',
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    search: {
      provider: 'local',
    },
    nav: [
      { text: 'Quick Start', link: '/guide/quick-start' },
    ],
    sidebar: [
      { text: 'Quick Start', link: '/guide/quick-start' },
      { text: 'Installation', link: '/guide/installation' },
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/petr-korobeinikov/claude-shim' },
    ],
  },
})
