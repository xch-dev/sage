// to generate json schema
// typescript-json-schema src/lib/theme.type.ts Theme --required > ./src/themes/schema.json
export interface Theme {
  name: string;
  displayName: string;
  schemaVersion: number; // 1 is the current version
  inherits?: string;
  most_like?: 'light' | 'dark';
  icon_path?: string;
  backgroundImage?: string;
  backgroundSize?: string;
  backgroundPosition?: string;
  backgroundRepeat?: string;
  isUserTheme?: boolean; // this is set at runtime by the loader
  colors?: {
    background?: string;
    backgroundTransparent?: string;
    foreground?: string;
    card?: string;
    cardTransparent?: string;
    cardForeground?: string;
    popover?: string;
    popoverTransparent?: string;
    popoverForeground?: string;
    primary?: string;
    primaryForeground?: string;
    secondary?: string;
    secondaryForeground?: string;
    muted?: string;
    mutedForeground?: string;
    accent?: string;
    accentForeground?: string;
    destructive?: string;
    destructiveForeground?: string;
    border?: string;
    input?: string;
    inputBackground?: string;
    ring?: string;
    cardBackdropFilter?: string;
    cardBackdropFilterWebkit?: string;
    popoverBackdropFilter?: string;
    popoverBackdropFilterWebkit?: string;
    inputBackdropFilter?: string;
    inputBackdropFilterWebkit?: string;
  };
  fonts?: {
    sans?: string;
    serif?: string;
    mono?: string;
    heading?: string;
    body?: string;
  };
  corners?: {
    none?: string;
    sm?: string;
    md?: string;
    lg?: string;
    xl?: string;
    full?: string;
  };
  shadows?: {
    none?: string;
    sm?: string;
    md?: string;
    lg?: string;
    xl?: string;
    inner?: string;
    card?: string;
    button?: string;
    dropdown?: string;
  };
  // Optional theme-specific sidebar configuration
  sidebar?: {
    background?: string;
    backdropFilter?: string;
    backdropFilterWebkit?: string;
    border?: string;
  };
  // Optional theme-specific table configurations
  tables?: {
    background?: string;
    border?: string;
    borderRadius?: string;
    boxShadow?: string;
    header?: {
      background?: string;
      color?: string;
      border?: string;
      fontWeight?: string;
      fontSize?: string;
      padding?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
    };
    row?: {
      background?: string;
      color?: string;
      border?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
      };
      selected?: {
        background?: string;
        color?: string;
      };
    };
    cell?: {
      padding?: string;
      border?: string;
      fontSize?: string;
    };
    footer?: {
      background?: string;
      color?: string;
      border?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
    };
  };
  // Optional theme-specific button configurations
  buttons?: {
    default?: {
      background?: string;
      color?: string;
      border?: string;
      borderStyle?: string;
      borderWidth?: string;
      borderColor?: string;
      borderRadius?: string;
      boxShadow?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
      active?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
    };
    outline?: {
      background?: string;
      color?: string;
      border?: string;
      borderStyle?: string;
      borderWidth?: string;
      borderColor?: string;
      borderRadius?: string;
      boxShadow?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
      active?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
    };
    secondary?: {
      background?: string;
      color?: string;
      border?: string;
      borderStyle?: string;
      borderWidth?: string;
      borderColor?: string;
      borderRadius?: string;
      boxShadow?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
      active?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
    };
    destructive?: {
      background?: string;
      color?: string;
      border?: string;
      borderStyle?: string;
      borderWidth?: string;
      borderColor?: string;
      borderRadius?: string;
      boxShadow?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
      active?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
    };
    ghost?: {
      background?: string;
      color?: string;
      border?: string;
      borderStyle?: string;
      borderWidth?: string;
      borderColor?: string;
      borderRadius?: string;
      boxShadow?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
      active?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
    };
    link?: {
      background?: string;
      color?: string;
      border?: string;
      borderStyle?: string;
      borderWidth?: string;
      borderColor?: string;
      borderRadius?: string;
      boxShadow?: string;
      backdropFilter?: string;
      backdropFilterWebkit?: string;
      hover?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
      active?: {
        background?: string;
        color?: string;
        transform?: string;
        borderStyle?: string;
        borderColor?: string;
        boxShadow?: string;
      };
    };
  };
  // Button style flags for dynamic CSS application
  buttonStyles?: string[];
  // Optional theme-specific switch configurations
  switches?: {
    checked?: {
      background?: string;
    };
    unchecked?: {
      background?: string;
    };
    thumb?: {
      background?: string;
    };
  };
}
