import "./KpiCard.css";

type Props = {
    label: string;
    value: string | number;
};

export default function KpiCard({ label, value }: Props) {
    return (
        <div className="kpi">
            <span className="kpi-label">{label}</span>
            <strong className="kpi-value">{value}</strong>
        </div>
    );
}
