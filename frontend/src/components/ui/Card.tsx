import React from "react";
import { cn } from "../../utils/cn";

export interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: "default" | "glass" | "glass-strong" | "glow" | "compact";
  padding?: "none" | "sm" | "md" | "lg";
  hover?: boolean;
  glow?: "primary" | "success" | "danger" | "warning" | "info" | "none";
  animated?: boolean;
}

const Card = React.forwardRef<HTMLDivElement, CardProps>(
  (
    {
      className,
      variant = "default",
      padding = "md",
      hover = true,
      glow = "none",
      animated = true,
      children,
      ...props
    },
    ref,
  ) => {
    const paddingClasses = {
      none: "",
      sm: "p-4",
      md: "p-6",
      lg: "p-8",
    };

    const variantClasses = {
      default: "card",
      glass: "glass rounded-xl",
      "glass-strong": "glass-strong rounded-xl",
      glow: "glass rounded-xl border-glow",
      compact: "card-compact",
    };

    const glowClasses = {
      primary: "glow-primary",
      success: "glow-success",
      danger: "glow-danger",
      warning: "glow-warning",
      info: "glow-info",
      none: "",
    };

    return (
      <div
        ref={ref}
        className={cn(
          variantClasses[variant],
          paddingClasses[padding],
          hover && "hover:scale-[1.02] hover:shadow-card-hover",
          animated && "transition-all duration-300",
          glow !== "none" && glowClasses[glow],
          "group relative overflow-hidden",
          className,
        )}
        {...props}
      >
        {/* 背景装饰效果 */}
        {variant === "glow" && (
          <div className="absolute inset-0 bg-gradient-to-br from-primary/5 via-transparent to-success/5 pointer-events-none" />
        )}

        {/* 边框光效 */}
        {glow !== "none" && (
          <div
            className={cn(
              "absolute inset-0 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none",
              "bg-gradient-to-r from-transparent via-current to-transparent",
              "animate-glow-pulse",
            )}
          />
        )}

        {/* 内容区域 */}
        <div className="relative z-10">{children}</div>
      </div>
    );
  },
);

Card.displayName = "Card";

const CardHeader = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn(
      "flex flex-col space-y-1.5 pb-4",
      "border-b border-border/50",
      className,
    )}
    {...props}
  />
));

CardHeader.displayName = "CardHeader";

const CardTitle = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLHeadingElement>
>(({ className, ...props }, ref) => (
  <h3
    ref={ref}
    className={cn(
      "font-semibold leading-none tracking-tight text-foreground",
      "text-lg md:text-xl",
      "flex items-center gap-2",
      className,
    )}
    {...props}
  />
));

CardTitle.displayName = "CardTitle";

const CardDescription = React.forwardRef<
  HTMLParagraphElement,
  React.HTMLAttributes<HTMLParagraphElement>
>(({ className, ...props }, ref) => (
  <p
    ref={ref}
    className={cn("text-sm text-muted leading-relaxed", className)}
    {...props}
  />
));

CardDescription.displayName = "CardDescription";

const CardContent = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div ref={ref} className={cn("pt-0 space-y-4", className)} {...props} />
));

CardContent.displayName = "CardContent";

const CardFooter = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn(
      "flex items-center justify-between pt-4",
      "border-t border-border/50",
      className,
    )}
    {...props}
  />
));

CardFooter.displayName = "CardFooter";

// 特殊卡片变体
const StatCard = React.forwardRef<
  HTMLDivElement,
  CardProps & {
    title: string;
    value: string | number;
    change?: string;
    trend?: "up" | "down" | "neutral";
    icon?: React.ReactNode;
  }
>(
  (
    { title, value, change, trend = "neutral", icon, className, ...props },
    ref,
  ) => {
    const trendColors = {
      up: "text-success",
      down: "text-danger",
      neutral: "text-muted",
    };

    return (
      <Card
        ref={ref}
        variant="glass"
        glow={trend === "up" ? "success" : trend === "down" ? "danger" : "none"}
        className={cn("text-center", className)}
        {...props}
      >
        <CardContent className="space-y-3">
          {icon && (
            <div className="flex justify-center">
              <div className="p-2 rounded-lg bg-primary/10 text-primary">
                {icon}
              </div>
            </div>
          )}
          <div>
            <div className="data-value text-2xl md:text-3xl">{value}</div>
            <div className="data-label mt-1">{title}</div>
            {change && (
              <div className={cn("text-xs mt-2", trendColors[trend])}>
                {change}
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    );
  },
);

StatCard.displayName = "StatCard";

// 状态卡片
const StatusCard = React.forwardRef<
  HTMLDivElement,
  CardProps & {
    status: "online" | "offline" | "warning" | "error";
    title: string;
    description?: string;
  }
>(({ status, title, description, className, ...props }, ref) => {
  const statusConfig = {
    online: {
      glow: "success" as const,
      dot: "pulse-dot-success",
      text: "text-success",
    },
    offline: {
      glow: "none" as const,
      dot: "w-2 h-2 rounded-full bg-muted",
      text: "text-muted",
    },
    warning: {
      glow: "warning" as const,
      dot: "pulse-dot-warning",
      text: "text-warning",
    },
    error: {
      glow: "danger" as const,
      dot: "pulse-dot-danger",
      text: "text-danger",
    },
  };

  const config = statusConfig[status];

  return (
    <Card
      ref={ref}
      variant="glass"
      glow={config.glow}
      className={className}
      {...props}
    >
      <CardContent>
        <div className="flex items-center gap-3">
          <div className={config.dot} />
          <div className="flex-1">
            <div className={cn("font-medium", config.text)}>{title}</div>
            {description && (
              <div className="text-sm text-muted mt-1">{description}</div>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
});

StatusCard.displayName = "StatusCard";

export {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
  StatCard,
  StatusCard,
};
