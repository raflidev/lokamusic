---
name: Organic Precision
colors:
  surface: '#111316'
  surface-dim: '#111316'
  surface-bright: '#37393d'
  surface-container-lowest: '#0c0e11'
  surface-container-low: '#1a1c1f'
  surface-container: '#1e2023'
  surface-container-high: '#282a2d'
  surface-container-highest: '#333538'
  on-surface: '#e2e2e6'
  on-surface-variant: '#c1c8c7'
  inverse-surface: '#e2e2e6'
  inverse-on-surface: '#2f3034'
  outline: '#8b9292'
  outline-variant: '#414848'
  surface-tint: '#aecccc'
  primary: '#aecccc'
  on-primary: '#193535'
  primary-container: '#1e3a3a'
  on-primary-container: '#86a4a3'
  inverse-primary: '#476363'
  secondary: '#ffb77d'
  on-secondary: '#4d2600'
  secondary-container: '#d97707'
  on-secondary-container: '#432100'
  tertiary: '#e9bdab'
  on-tertiary: '#452a1d'
  tertiary-container: '#4b2f22'
  on-tertiary-container: '#be9685'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#cae8e8'
  primary-fixed-dim: '#aecccc'
  on-primary-fixed: '#022020'
  on-primary-fixed-variant: '#304c4b'
  secondary-fixed: '#ffdcc3'
  secondary-fixed-dim: '#ffb77d'
  on-secondary-fixed: '#2f1500'
  on-secondary-fixed-variant: '#6e3900'
  tertiary-fixed: '#ffdbcc'
  tertiary-fixed-dim: '#e9bdab'
  on-tertiary-fixed: '#2d150a'
  on-tertiary-fixed-variant: '#5e4032'
  background: '#111316'
  on-background: '#e2e2e6'
  surface-variant: '#333538'
typography:
  display-lg:
    fontFamily: Geist
    fontSize: 48px
    fontWeight: '300'
    lineHeight: '1.1'
    letterSpacing: -0.03em
  headline-lg:
    fontFamily: Geist
    fontSize: 32px
    fontWeight: '400'
    lineHeight: '1.2'
    letterSpacing: -0.02em
  headline-lg-mobile:
    fontFamily: Geist
    fontSize: 24px
    fontWeight: '400'
    lineHeight: '1.2'
    letterSpacing: -0.01em
  title-md:
    fontFamily: Geist
    fontSize: 20px
    fontWeight: '500'
    lineHeight: '1.4'
    letterSpacing: -0.01em
  body-lg:
    fontFamily: Geist
    fontSize: 16px
    fontWeight: '400'
    lineHeight: '1.6'
    letterSpacing: '0'
  body-md:
    fontFamily: Geist
    fontSize: 14px
    fontWeight: '400'
    lineHeight: '1.5'
    letterSpacing: '0'
  label-sm:
    fontFamily: Geist
    fontSize: 12px
    fontWeight: '600'
    lineHeight: '1'
    letterSpacing: 0.05em
rounded:
  sm: 0.125rem
  DEFAULT: 0.25rem
  md: 0.375rem
  lg: 0.5rem
  xl: 0.75rem
  full: 9999px
spacing:
  unit: 8px
  container-max: 1280px
  gutter: 24px
  margin-mobile: 16px
  margin-desktop: 40px
---

## Brand & Style

The design system moves away from the ephemeral "AI-glow" aesthetic toward a grounded, editorial experience that feels permanent and tactile. The target audience includes professionals, designers, and curators who value substance over spectacle. 

The visual style is a blend of **Modern Minimalism** and **Tactile Depth**. It uses heavy whitespace and a restricted, nature-inspired palette to create an atmosphere of quiet confidence. Instead of digital neon and synthetic blurs, it leans into the aesthetics of high-end industrial design and premium print media—think brushed metals, matte papers, and subtle, physical layering. The emotional response is one of calm focus, reliability, and timelessness.

## Colors

This design system utilizes a dark-mode-first approach grounded in **Warm Slates** and **Charcoal Greys**, avoiding the clinical feel of pure black. 

The primary accent is a **Deep Teal (#1E3A3A)**, used for structural emphasis and primary actions, providing an organic, forest-like depth. A **Muted Amber (#D97706)** serves as the secondary accent, reserved for critical highlights, notifications, or specific calls to action that require warmth and visibility without being jarring. 

Gradients should be extremely subtle, transitioning between varying shades of slate to suggest curved surfaces or soft, directional light rather than digital luminescence.

## Typography

The typography leverages the technical precision of **Geist** but applies it with an **editorial sensibility**. Headlines utilize lighter weights (`300` and `400`) with tight letter-spacing to mimic high-end architectural journals. 

Hierarchy is established through weight shifts and intentional whitespace rather than drastic size changes. Body text is optimized for long-form legibility with a generous line height (`1.6`), while labels use bold, all-caps styling to provide a clear, functional anchor to the page layout.

## Layout & Spacing

The layout follows a **Fixed Grid** philosophy for desktop to maintain an editorial, "contained" feel, while transitioning to a fluid model for mobile.

The spacing rhythm is based on an **8px linear scale**. Use large internal paddings (32px+) for containers to emphasize the feeling of luxury and space. Elements should be aligned to a 12-column grid on desktop, with gutters of 24px. On mobile, margins reduce to 16px to maximize content area, and columns collapse into a single vertical stack.

## Elevation & Depth

Depth is achieved through **Tonal Layers** and **Ambient Shadows** rather than transparency. Instead of floating elements on top of a blur, the design system uses "Surface-on-Surface" stacking. 

Shadows are exceptionally soft, with a large blur radius and low opacity (approx. 15-20%), tinted with the charcoal background color to feel like natural occlusion shadows. Subtle 1px inner-borders (top-down light source) can be used on primary containers to give them a beveled, machined appearance. This creates a tactile, physical depth that feels carved rather than projected.

## Shapes

The shape language is **Soft and Structural**. A `0.25rem` (4px) corner radius is the standard for most components, providing just enough softness to feel approachable while maintaining the discipline of a professional tool. 

Large containers and cards use `0.5rem` (8px) to define major sections. Avoid pill-shaped buttons unless used for secondary, low-priority tags. The geometry should feel intentional and rectangular, echoing the grid-based layout.

## Components

### Buttons
Primary buttons use a solid Deep Teal fill with high-contrast white text. Secondary buttons utilize a subtle charcoal outline with no fill. All buttons feature a 1px top-edge highlight to simulate a physical button cap.

### Input Fields
Inputs are grounded with a solid background slightly darker than the surface they sit on. Active states are indicated by a 1px Muted Amber bottom border rather than a full focus ring, maintaining a minimalist profile.

### Cards
Cards do not use heavy shadows. Instead, they are defined by a subtle 1px border (`#282C33`) and a slight background color shift from the main canvas. This keeps the interface flat and sophisticated.

### Lists & Data
Lists should use generous vertical padding. Dividers are low-contrast and never span the full width of the container, creating a "breathable" list structure.

### Chips & Tags
Tags are rectangular with sharp 2px corners. They use a monochromatic palette (dark grey background, light grey text) to avoid competing with primary action buttons.