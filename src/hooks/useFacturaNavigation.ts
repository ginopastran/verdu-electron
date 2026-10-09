import { useNavigate } from "react-router-dom";

export const useFacturaNavigation = () => {
  const navigate = useNavigate();

  const goToFacturaDetalle = (facturaId: string) => {
    navigate(`/facturas/${facturaId}`);
  };

  const goToFacturasList = () => {
    navigate("/facturas");
  };

  const goBack = () => {
    navigate(-1);
  };

  return {
    goToFacturaDetalle,
    goToFacturasList,
    goBack,
  };
};
