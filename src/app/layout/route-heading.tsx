export function RouteHeading({
  eyebrow,
  title,
  detail,
}: {
  eyebrow: string;
  title: string;
  detail?: string;
}) {
  return (
    <header className="route-heading">
      <p className="eyebrow">{eyebrow}</p>
      <h1 data-route-heading tabIndex={-1}>
        {title}
      </h1>
      {detail ? <p className="route-heading-detail">{detail}</p> : null}
    </header>
  );
}
