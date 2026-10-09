import { useBusiness } from "@/contexts/BusinessContext";

export const useBusinessInfo = (_API_URL?: string, _appId?: string | null) => {
  const { businessInfo, businessInfoLoading, loading } = useBusiness();

  return {
    businessInfo,
    loading: loading || businessInfoLoading,
  };
};
