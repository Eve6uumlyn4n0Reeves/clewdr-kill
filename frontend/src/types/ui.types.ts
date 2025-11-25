export interface StatusMessageProps {
  type: "success" | "error" | "warning" | "info";
  message: string;
}

export interface ButtonProps {
  type?: "button" | "submit" | "reset";
  onClick?: () => void;
  disabled?: boolean;
  isLoading?: boolean;
  variant?: "primary" | "secondary" | "danger" | "success";
  size?: "sm" | "md" | "lg";
  className?: string;
  children: React.ReactNode;
}

export interface FormInputProps {
  id: string;
  name: string;
  type?: string;
  value: string;
  onChange: (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>,
  ) => void;
  label?: string;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
  isTextarea?: boolean;
  rows?: number;
  error?: string;
  onClear?: () => void;
  min?: string | number;
  max?: string | number;
}

export interface CardProps {
  children: React.ReactNode;
  className?: string;
}
