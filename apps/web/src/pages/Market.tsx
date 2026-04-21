import { useMarketPage } from '../features/market/useMarketPage';

import { MarketContent } from './market/MarketContent';

export default function Market() {
  const state = useMarketPage();

  return <MarketContent state={state} />;
}
