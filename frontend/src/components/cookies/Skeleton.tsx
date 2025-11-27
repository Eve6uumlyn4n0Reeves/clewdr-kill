const CookieTableSkeleton = () => (
  <div className="animate-pulse space-y-2">
    {Array.from({ length: 6 }).map((_, i) => (
      <div
        key={i}
        className="h-12 rounded bg-surfaceHighlight/60 dark:bg-gray-800 border border-border/50"
      />
    ))}
  </div>
);

export default CookieTableSkeleton;
