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
      {
        text: 'Guide',
        items: [
          { text: 'Quick Start', link: '/guide/quick-start' },
          { text: 'Installation', link: '/guide/installation' },
          { text: 'Prompt indicator', link: '/guide/prompt-indicator' },
          { text: 'Profiles', link: '/guide/profiles' },
          { text: 'Profile resolution', link: '/guide/resolution' },
          { text: 'Migration', link: '/guide/migration' },
        ],
      },
      { text: 'Contributing', link: '/contributing' },
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/petr-korobeinikov/claude-shim' },
    ],
  },
})
