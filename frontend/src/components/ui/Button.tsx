import React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../utils/cn";

const buttonVariants = cva(
  "inline-flex items-center justify-center font-medium rounded-lg transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-background disabled:opacity-50 disabled:pointer-events-none relative overflow-hidden group",
  {
    variants: {
      variant: {
        primary: [
          "btn-primary",
          "bg-primary text-white hover:bg-primary-600",
          "shadow-glow-sm shadow-primary-glow",
          "hover:shadow-glow-md hover:shadow-primary-glow",
          "hover:-translate-y-0.5",
          "active:translate-y-0",
        ],
        secondary: [
          "btn-secondary",
          "bg-buttonSecondary text-foreground hover:bg-buttonSecondaryHover",
          "border border-border hover:border-borderHighlight",
          "hover:-translate-y-0.5",
          "hover:shadow-dark-md",
        ],
        ghost: [
          "btn-ghost",
          "text-foreground hover:bg-surfaceHighlight",
          "hover:text-primary",
          "hover:shadow-inner-glow hover:shadow-primary-glow",
        ],
        danger: [
          "btn-danger",
          "bg-danger text-white hover:bg-danger-600",
          "shadow-glow-sm shadow-danger-glow",
          "hover:shadow-glow-md hover:shadow-danger-glow",
          "hover:-translate-y-0.5",
        ],
        success: [
          "bg-success text-white hover:bg-success-600",
          "shadow-glow-sm shadow-success-glow",
          "hover:shadow-glow-md hover:shadow-success-glow",
          "hover:-translate-y-0.5",
        ],
        warning: [
          "bg-warning text-white hover:bg-warning-600",
          "shadow-glow-sm shadow-warning-glow",
          "hover:shadow-glow-md hover:shadow-warning-glow",
          "hover:-translate-y-0.5",
        ],
        info: [
          "bg-info text-white hover:bg-info-600",
          "shadow-glow-sm shadow-info-glow",
          "hover:shadow-glow-md hover:shadow-info-glow",
          "hover:-translate-y-0.5",
        ],
        outline: [
          "border-2 border-primary text-primary hover:bg-primary hover:text-white",
          "hover:shadow-glow-md hover:shadow-primary-glow",
          "hover:-translate-y-0.5",
        ],
        link: [
          "text-primary hover:text-primary-400 underline-offset-4 hover:underline",
          "p-0 h-auto font-normal",
        ],
      },
      size: {
        xs: "px-2 py-1 text-xs h-7",
        sm: "px-3 py-1.5 text-xs h-8",
        md: "px-4 py-2 text-sm h-10",
        lg: "px-6 py-3 text-base h-12",
        xl: "px-8 py-4 text-lg h-14",
        icon: "h-10 w-10 p-0",
        "icon-sm": "h-8 w-8 p-0",
        "icon-lg": "h-12 w-12 p-0",
      },
      fullWidth: {
        true: "w-full",
        false: "",
      },
    },
    defaultVariants: {
      variant: "primary",
      size: "md",
      fullWidth: false,
    },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  loading?: boolean;
  icon?: React.ReactNode;
  iconPosition?: "left" | "right";
  pulse?: boolean;
  glow?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      className,
      variant,
      size,
      fullWidth,
      loading,
      children,
      disabled,
      icon,
      iconPosition = "left",
      pulse = false,
      glow = false,
      ...props
    },
    ref,
  ) => {
    const isIconOnly = !children && icon;
    const actualSize = isIconOnly
      ? size === "sm"
        ? "icon-sm"
        : size === "lg"
          ? "icon-lg"
          : "icon"
      : size;

    return (
      <button
        className={cn(
          buttonVariants({ variant, size: actualSize, fullWidth }),
          pulse && "animate-pulse-slow",
          glow && "animate-glow-pulse",
          loading && "cursor-wait",
          className,
        )}
        ref={ref}
        disabled={disabled || loading}
        {...props}
      >
        {/* 背景光效 */}
        {(variant === "primary" ||
          variant === "danger" ||
          variant === "success") && (
          <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent -translate-x-full group-hover:translate-x-full transition-transform duration-700" />
        )}

        {/* 加载动画 */}
        {loading && (
          <div className="loading-spinner w-4 h-4 mr-2 flex-shrink-0" />
        )}

        {/* 左侧图标 */}
        {icon && !loading && iconPosition === "left" && (
          <span className={cn("flex-shrink-0", children && "mr-2")}>
            {icon}
          </span>
        )}

        {/* 按钮文本 */}
        {children && (
          <span className="relative z-10 font-medium">{children}</span>
        )}

        {/* 右侧图标 */}
        {icon && !loading && iconPosition === "right" && (
          <span className={cn("flex-shrink-0", children && "ml-2")}>
            {icon}
          </span>
        )}
      </button>
    );
  },
);

Button.displayName = "Button";

// 特殊按钮变体
const IconButton = React.forwardRef<
  HTMLButtonElement,
  Omit<ButtonProps, "children"> & {
    icon: React.ReactNode;
    tooltip?: string;
  }
>(({ icon, tooltip, className, ...props }, ref) => (
  <Button ref={ref} className={cn("relative group", className)} {...props}>
    {icon}
    {tooltip && (
      <div className="tooltip absolute -top-10 left-1/2 -translate-x-1/2 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none whitespace-nowrap">
        {tooltip}
      </div>
    )}
  </Button>
));

IconButton.displayName = "IconButton";

// 浮动操作按钮
const FloatingActionButton = React.forwardRef<
  HTMLButtonElement,
  ButtonProps & {
    position?: "bottom-right" | "bottom-left" | "top-right" | "top-left";
  }
>(({ position = "bottom-right", className, ...props }, ref) => {
  const positionClasses = {
    "bottom-right": "fixed bottom-6 right-6",
    "bottom-left": "fixed bottom-6 left-6",
    "top-right": "fixed top-6 right-6",
    "top-left": "fixed top-6 left-6",
  };

  return (
    <Button
      ref={ref}
      size="icon-lg"
      className={cn(
        positionClasses[position],
        "rounded-full shadow-lg z-50",
        "animate-float",
        className,
      )}
      {...props}
    />
  );
});

FloatingActionButton.displayName = "FloatingActionButton";

// 按钮组
const ButtonGroup = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & {
    orientation?: "horizontal" | "vertical";
    attached?: boolean;
  }
>(
  (
    { className, orientation = "horizontal", attached = false, ...props },
    ref,
  ) => (
    <div
      ref={ref}
      className={cn(
        "flex",
        orientation === "horizontal" ? "flex-row" : "flex-col",
        attached &&
          orientation === "horizontal" &&
          "[&>button:not(:first-child)]:rounded-l-none [&>button:not(:last-child)]:rounded-r-none [&>button:not(:first-child)]:-ml-px",
        attached &&
          orientation === "vertical" &&
          "[&>button:not(:first-child)]:rounded-t-none [&>button:not(:last-child)]:rounded-b-none [&>button:not(:first-child)]:-mt-px",
        !attached && "gap-2",
        className,
      )}
      {...props}
    />
  ),
);

ButtonGroup.displayName = "ButtonGroup";

export { Button, IconButton, FloatingActionButton, ButtonGroup };
export default Button;
