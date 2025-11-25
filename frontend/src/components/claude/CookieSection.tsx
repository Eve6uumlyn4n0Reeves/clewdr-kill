import React from "react";
import { useTranslation } from "react-i18next";
import type { CookieItem } from "../../types/cookie.types";

interface CookieSectionProps {
  title: string;
  cookies: CookieItem[];
  color: string;
  renderStatus: (item: CookieItem, index: number) => React.ReactNode;
}

const CookieSection: React.FC<CookieSectionProps> = ({
  title,
  cookies,
  color,
  renderStatus,
}) => {
  const { t } = useTranslation();
  const colorClasses = (() => {
    switch (color) {
      case "yellow":
        return {
          bg: "bg-yellow-900",
          border: "border-yellow-700",
          text: "text-yellow-100",
          pillBg: "bg-yellow-800",
        };
      case "red":
        return {
          bg: "bg-red-900",
          border: "border-red-700",
          text: "text-red-100",
          pillBg: "bg-red-800",
        };
      case "green":
        return {
          bg: "bg-green-900",
          border: "border-green-700",
          text: "text-green-100",
          pillBg: "bg-green-800",
        };
      default:
        return {
          bg: "bg-blue-900",
          border: "border-blue-700",
          text: "text-blue-100",
          pillBg: "bg-blue-800",
        };
    }
  })();
  // sort cookie base on reset_time
  cookies.sort((a, b) => {
    const aTime = a.reset_time ? new Date(a.reset_time).getTime() : 0;
    const bTime = b.reset_time ? new Date(b.reset_time).getTime() : 0;
    return aTime - bTime;
  });

  return (
    <div className={`rounded-lg bg-gray-800 overflow-hidden w-full shadow-md`}>
      <div
        className={`${colorClasses.bg} px-4 py-2 flex justify-between items-center border-b ${colorClasses.border}`}
      >
        <h4 className={`font-medium ${colorClasses.text}`}>{title}</h4>
        <span
          className={`${colorClasses.pillBg} ${colorClasses.text} text-xs px-2 py-1 rounded-full`}
        >
          {cookies.length}
        </span>
      </div>
      {cookies.length > 0 ? (
        <div className="p-4 divide-y divide-gray-700">
          {cookies.map((item, index) => renderStatus(item, index))}
        </div>
      ) : (
        <div className="p-4 text-sm text-gray-400 italic">
          {t("cookieStatus.noCookies", { type: title.toLowerCase() })}
        </div>
      )}
    </div>
  );
};

export default CookieSection;
